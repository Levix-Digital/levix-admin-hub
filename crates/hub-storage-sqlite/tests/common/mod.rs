//! A Hub on a temporary database with real cryptography and a clock the test moves.

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use data_encoding::BASE32_NOPAD;
use hub_core::context::Context;
use hub_core::credential::LinkCompletion;
use hub_core::organization::{NewOrganization, Organization};
use hub_core::ports::Clock;
use hub_core::role::Role;
use hub_core::session::{SessionKind, SignIn};
use hub_core::Hub;
use hub_crypto::{key_file, HubCrypto};
use hub_storage_sqlite::SqliteStore;
use time::{Duration, OffsetDateTime};

pub const PASSWORD: &str = "correct horse battery staple";

pub fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

#[derive(Debug)]
pub struct TestClock(Mutex<OffsetDateTime>);

impl TestClock {
    pub fn advance(&self, by: Duration) {
        let mut now = self.0.lock().unwrap_or_else(|e| e.into_inner());
        *now += by;
    }
}

impl Clock for TestClock {
    fn now(&self) -> OffsetDateTime {
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub type TestHub = Hub<SqliteStore, HubCrypto, TestClock>;

pub struct Fixture {
    pub _dir: tempfile::TempDir,
    pub hub: TestHub,
    pub clock: Arc<TestClock>,
}

/// An administrator who completed their link, with the TOTP secret their app would hold.
pub struct Account {
    pub email: String,
    pub secret: Vec<u8>,
    pub context: Context,
    pub recovery_codes: Vec<String>,
}

impl Fixture {
    pub async fn new() -> Self {
        let dir = ok(tempfile::tempdir());
        let store = ok(SqliteStore::open(&dir.path().join("hub.db")).await);
        let key = ok(key_file::create(&dir.path().join("secret.key")));
        let clock = Arc::new(TestClock(Mutex::new(time::macros::datetime!(
            2026-11-06 12:00:00 UTC
        ))));
        let hub = Hub::new(store, Arc::new(HubCrypto::new(&key)), Arc::clone(&clock));
        Self {
            _dir: dir,
            hub,
            clock,
        }
    }

    /// The code an authenticator shows now. Moves the clock one step first, so two calls
    /// never return the same step (replay).
    pub fn code(&self, secret: &[u8]) -> String {
        self.clock.advance(Duration::seconds(30));
        self.current_code(secret)
    }

    /// The code an authenticator shows now, without moving the clock.
    pub fn current_code(&self, secret: &[u8]) -> String {
        let step = self.clock.now().unix_timestamp().div_euclid(30);
        hub_crypto::totp::code_at_step(secret, step).unwrap_or_default()
    }

    pub fn anonymous(&self) -> Context {
        Context::anonymous(Some("192.0.2.10".to_owned()))
    }

    /// Completes a link as its holder would: enrollment, then password and code.
    pub async fn complete(&self, token: &str, email: &str) -> Account {
        let anonymous = self.anonymous();
        let enrollment = ok(self.hub.begin_enrollment(&anonymous, token).await);
        let secret = ok(BASE32_NOPAD.decode(enrollment.secret.as_bytes()));
        let code = self.code(&secret);
        let enrolled = ok(self
            .hub
            .complete_link(
                &anonymous,
                LinkCompletion {
                    token,
                    display_name: None,
                    password: PASSWORD,
                    code: &code,
                },
                SessionKind::Cli,
                None,
            )
            .await);
        let (context, _) = ok(self
            .hub
            .authenticate_session(&enrolled.session.token, None)
            .await);
        Account {
            email: email.to_owned(),
            secret,
            context,
            recovery_codes: enrolled.recovery_codes,
        }
    }

    /// Initializes the Hub and claims it.
    pub async fn owner(&self) -> Account {
        let (_, link) = ok(self
            .hub
            .initialize(&Context::host(), "owner@example.com")
            .await);
        self.complete(&link.token, "owner@example.com").await
    }

    pub async fn organization(&self, owner: &Account, slug: &str) -> Organization {
        ok(self
            .hub
            .create_organization(
                &owner.context,
                NewOrganization {
                    slug: slug.to_owned(),
                    name: slug.to_uppercase(),
                },
            )
            .await)
    }

    /// Invites `email` with `role` and completes the invitation.
    pub async fn member(
        &self,
        inviter: &Account,
        organization: &Organization,
        email: &str,
        role: Role,
    ) -> Account {
        let added = ok(self
            .hub
            .add_member(&inviter.context, organization.id, email, role)
            .await);
        let link = added.invitation.unwrap_or_else(|| panic!("no invitation"));
        self.complete(&link.token, email).await
    }

    /// Signs in again and returns the fresh context (roles read now).
    pub async fn refresh(&self, account: &Account) -> Context {
        let code = self.code(&account.secret);
        match ok(self
            .hub
            .sign_in(
                &self.anonymous(),
                &account.email,
                PASSWORD,
                &code,
                SessionKind::Cli,
                None,
            )
            .await)
        {
            SignIn::Session(issued) => {
                ok(self.hub.authenticate_session(&issued.token, None).await).0
            }
            SignIn::MustReenroll(_) => panic!("unexpected re-enrollment"),
        }
    }
}

impl Fixture {
    pub fn clock_elapsed_since(&self, start: OffsetDateTime) -> Duration {
        self.clock.now() - start
    }

    /// A context from signing in, or the anonymous context if sign-in is refused.
    pub async fn refresh_context(&self, account: &Account) -> Context {
        let code = self.code(&account.secret);
        match self
            .hub
            .sign_in(
                &self.anonymous(),
                &account.email,
                PASSWORD,
                &code,
                SessionKind::Cli,
                None,
            )
            .await
        {
            Ok(SignIn::Session(issued)) => self
                .hub
                .authenticate_session(&issued.token, None)
                .await
                .map(|(context, _)| context)
                .unwrap_or_else(|_| self.anonymous()),
            _ => self.anonymous(),
        }
    }
}
