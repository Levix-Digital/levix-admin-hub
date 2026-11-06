//! Single-use credential links: the setup link of the instance owner, invitations and
//! recovery. Whoever holds a valid link sets a password and enrolls TOTP (FR-001, FR-005).

use serde::Serialize;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::administrator::{
    check_password, normalize_display_name, Administrator, AdministratorStatus,
};
use crate::audit::hex;
use crate::context::Context;
use crate::hub::Hub;
use crate::ports::{new_id, Clock, Crypto, Store};
use crate::session::{IssuedSession, SessionKind};
use crate::{CoreError, CoreResult};

/// Issuer shown in authenticator apps.
pub const TOTP_ISSUER: &str = "Levix Admin Hub";
/// Number of recovery codes issued at enrollment.
pub const RECOVERY_CODES: usize = 10;

/// Why a link exists, which sets its validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkPurpose {
    /// The instance owner's first credentials (24 hours).
    Setup,
    /// A new administrator (72 hours).
    Invitation,
    /// New credentials after a reset or a recovery code (72 hours; 24 from the host).
    Recovery,
}

impl LinkPurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Invitation => "invitation",
            Self::Recovery => "recovery",
        }
    }

    pub fn parse(value: &str) -> CoreResult<Self> {
        match value {
            "setup" => Ok(Self::Setup),
            "invitation" => Ok(Self::Invitation),
            "recovery" => Ok(Self::Recovery),
            other => Err(CoreError::Storage(format!(
                "unknown link purpose '{other}'"
            ))),
        }
    }

    fn validity(self) -> Duration {
        match self {
            Self::Setup => Duration::hours(24),
            Self::Invitation | Self::Recovery => Duration::hours(72),
        }
    }
}

/// A stored link; only the hash of its token is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialLink {
    pub id: Uuid,
    pub administrator_id: Uuid,
    pub purpose: LinkPurpose,
    pub token_hash: String,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub used_at: Option<OffsetDateTime>,
}

/// A link just created, with its token (shown once, never stored).
#[derive(Debug, Clone)]
pub struct IssuedLink {
    pub link: CredentialLink,
    pub token: String,
}

/// What the holder of a valid link may see before using it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinkInfo {
    pub purpose: LinkPurpose,
    pub email: String,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
}

/// A TOTP secret to scan, returned once per enrollment start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TotpEnrollment {
    /// Base32 secret for manual entry.
    pub secret: String,
    /// `otpauth://` URI for a QR code.
    pub uri: String,
}

/// What the holder of a link submits to complete it.
#[derive(Debug, Clone, Copy)]
pub struct LinkCompletion<'a> {
    pub token: &'a str,
    pub display_name: Option<&'a str>,
    pub password: &'a str,
    /// Code from the authenticator, for the secret of `begin_enrollment`.
    pub code: &'a str,
}

/// Result of completing a link: a session and the recovery codes, shown once.
#[derive(Debug)]
pub struct Enrolled {
    pub session: IssuedSession,
    pub recovery_codes: Vec<String>,
}

/// SHA-256 of a token in lowercase hex: tokens are stored only this way.
pub fn token_hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (b as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// `otpauth://` URI understood by authenticator apps.
pub fn otpauth_uri(secret_base32: &str, account: &str) -> String {
    format!(
        "otpauth://totp/{issuer}:{account}?secret={secret_base32}&issuer={issuer}&algorithm=SHA1&digits=6&period=30",
        issuer = percent_encode(TOTP_ISSUER),
        account = percent_encode(account),
    )
}

impl<S: Store, K: Crypto, C: Clock> Hub<S, K, C> {
    pub(crate) fn new_link(
        &self,
        administrator: Uuid,
        purpose: LinkPurpose,
        created_by: Option<Uuid>,
        now: OffsetDateTime,
    ) -> IssuedLink {
        let token = self.crypto.token("lvxl_");
        IssuedLink {
            link: CredentialLink {
                id: new_id(),
                administrator_id: administrator,
                purpose,
                token_hash: token_hash(&token),
                created_by,
                created_at: now,
                expires_at: now + purpose.validity(),
                used_at: None,
            },
            token,
        }
    }

    /// A valid link and its administrator, or `NotFound` (unknown, used or expired links
    /// look the same, so a token cannot be probed).
    async fn valid_link(&self, token: &str) -> CoreResult<(CredentialLink, Administrator)> {
        let link = self
            .store
            .find_link(&token_hash(token))
            .await?
            .filter(|l| l.used_at.is_none() && l.expires_at > self.clock.now())
            .ok_or(CoreError::NotFound("link"))?;
        let administrator = self.administrator_by_id(link.administrator_id).await?;
        if administrator.status == AdministratorStatus::Suspended {
            return Err(CoreError::NotFound("link"));
        }
        Ok((link, administrator))
    }

    async fn refuse_link(&self, context: &Context, error: CoreError) -> CoreError {
        if matches!(error, CoreError::NotFound(_)) {
            self.record_refusal(context, "link.use", None, error).await
        } else {
            error
        }
    }

    /// What a valid link is for.
    pub async fn inspect_link(&self, context: &Context, token: &str) -> CoreResult<LinkInfo> {
        match self.valid_link(token).await {
            Ok((link, administrator)) => Ok(LinkInfo {
                purpose: link.purpose,
                email: administrator.email,
                expires_at: link.expires_at,
            }),
            Err(error) => Err(self.refuse_link(context, error).await),
        }
    }

    /// Starts TOTP enrollment for the holder of a valid link: a new pending secret replaces
    /// any earlier pending one; the confirmed secret, if any, keeps working until completion.
    pub async fn begin_enrollment(
        &self,
        context: &Context,
        token: &str,
    ) -> CoreResult<TotpEnrollment> {
        let (_, mut administrator) = match self.valid_link(token).await {
            Ok(found) => found,
            Err(error) => return Err(self.refuse_link(context, error).await),
        };
        let secret = self.crypto.new_secret();
        administrator.totp_pending_sealed = Some(self.crypto.seal(&secret)?);
        self.store
            .update_administrator(&administrator, None)
            .await?;
        let encoded = self.crypto.encode_secret(&secret);
        Ok(TotpEnrollment {
            uri: otpauth_uri(&encoded, &administrator.email),
            secret: encoded,
        })
    }

    /// Completes a link: checks the TOTP code against the pending secret, sets the password
    /// and display name, issues recovery codes, ends other sessions and signs in.
    pub async fn complete_link(
        &self,
        context: &Context,
        input: LinkCompletion<'_>,
        kind: SessionKind,
        user_agent: Option<&str>,
    ) -> CoreResult<Enrolled> {
        const ACTION: &str = "link.complete";
        let LinkCompletion {
            token,
            display_name,
            password,
            code,
        } = input;
        let (link, mut administrator) = match self.valid_link(token).await {
            Ok(found) => found,
            Err(error) => return Err(self.refuse_link(context, error).await),
        };
        let now = self.clock.now();
        let pending = administrator
            .totp_pending_sealed
            .as_deref()
            .ok_or_else(|| CoreError::Conflict("start the TOTP enrollment first".to_owned()))?;
        let secret = self.crypto.open(pending)?;
        let step = self
            .crypto
            .matching_step(&secret, code, now)
            .ok_or(CoreError::Invalid {
                field: "code",
                reason: "does not match the authenticator; check the device clock".to_owned(),
            })?;
        check_password(password, &administrator.email)?;
        if let Some(name) = display_name {
            administrator.display_name = normalize_display_name(name)?;
        }
        administrator.password_hash = Some(self.crypto.hash(password)?);
        administrator.totp_secret_sealed = administrator.totp_pending_sealed.take();
        administrator.totp_confirmed_at = Some(now);
        administrator.totp_last_step = Some(step);
        administrator.status = AdministratorStatus::Active;
        administrator.reset_failures();

        let recovery_codes: Vec<String> = (0..RECOVERY_CODES)
            .map(|_| self.crypto.recovery_code())
            .collect();
        let hashes: Vec<String> = recovery_codes
            .iter()
            .map(|c| token_hash(&normalize_recovery_code(c)))
            .collect();
        let instance = self.instance().await?;
        let session = self.new_session(
            &administrator,
            kind,
            &instance.settings,
            context,
            user_agent,
        );
        let audit = self
            .event(context, ACTION)
            .target("administrator", administrator.id)
            .detail("purpose", link.purpose.as_str());
        self.store
            .complete_link(&link, &administrator, &hashes, &session.session, &audit)
            .await?;
        Ok(Enrolled {
            session,
            recovery_codes,
        })
    }
}

/// Recovery codes compare without separators or case.
pub fn normalize_recovery_code(code: &str) -> String {
    code.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_is_escaped() {
        let uri = otpauth_uri("JBSWY3DPEHPK3PXP", "ana+it@example.com");
        assert!(uri.starts_with("otpauth://totp/Levix%20Admin%20Hub:ana%2Bit%40example.com?"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXP"));
    }

    #[test]
    fn recovery_codes_normalize() {
        assert_eq!(normalize_recovery_code("AB12-cd34-EF56"), "ab12cd34ef56");
    }

    #[test]
    fn hashes_are_stable_hex() {
        assert_eq!(token_hash("abc").len(), 64);
        assert_eq!(token_hash("abc"), token_hash("abc"));
    }
}
