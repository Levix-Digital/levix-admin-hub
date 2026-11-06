//! Use cases of FEAT-019 end to end through the core, the SQLite adapter and the real
//! cryptography: claim, sign-in, TOTP, lockout, roles, sessions, API tokens and audit.

mod common;

use common::{ok, Fixture, PASSWORD};
use hub_core::audit::AuditResult;
use hub_core::context::Context;
use hub_core::credential::LinkCompletion;
use hub_core::instance::InstanceSettings;
use hub_core::ports::AuditRepository;
use hub_core::role::Role;
use hub_core::session::{check_csrf, SessionKind, SignIn};
use hub_core::CoreError;
use time::Duration;

#[tokio::test]
async fn the_claimed_owner_signs_in_with_password_and_totp() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    assert_eq!(owner.recovery_codes.len(), 10);
    let me = ok(fixture.hub.me(&owner.context).await);
    assert!(me.administrator.is_instance_owner);
    assert!(me.administrator.second_factor_enrolled);

    let code = fixture.code(&owner.secret);
    let sign_in = |code: String| {
        let hub = fixture.hub.clone();
        let anonymous = fixture.anonymous();
        async move {
            hub.sign_in(
                &anonymous,
                "Owner@Example.com",
                PASSWORD,
                &code,
                SessionKind::Console,
                None,
            )
            .await
        }
    };
    let SignIn::Session(issued) = ok(sign_in(code.clone()).await) else {
        panic!("expected a session");
    };
    assert!(issued.csrf_token.is_some());
    // The same code (same time step) is refused: replay.
    assert_eq!(sign_in(code).await.err(), Some(CoreError::Unauthenticated));
}

#[tokio::test]
async fn wrong_email_password_or_code_fail_alike_and_are_audited() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let anonymous = fixture.anonymous();
    for (email, password, code) in [
        ("nobody@example.com", PASSWORD, fixture.code(&owner.secret)),
        (
            "owner@example.com",
            "wrong password here",
            fixture.code(&owner.secret),
        ),
        ("owner@example.com", PASSWORD, "000000".to_owned()),
    ] {
        let result = fixture
            .hub
            .sign_in(&anonymous, email, password, &code, SessionKind::Cli, None)
            .await;
        assert_eq!(result.err(), Some(CoreError::Unauthenticated));
    }
    let records = ok(fixture.hub.store.all_records().await);
    let denials = records
        .iter()
        .filter(|r| r.event.action == "auth.sign_in" && r.event.result == AuditResult::Denied)
        .count();
    assert_eq!(denials, 3);
    assert!(records
        .iter()
        .all(|r| !r.event.details.values().any(|v| v.contains(PASSWORD))));
}

#[tokio::test]
async fn repeated_failures_lock_the_account_for_a_growing_delay() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let anonymous = fixture.anonymous();
    let attempt = |password: &'static str| {
        let hub = fixture.hub.clone();
        let anonymous = anonymous.clone();
        let code = fixture.current_code(&owner.secret);
        async move {
            hub.sign_in(
                &anonymous,
                "owner@example.com",
                password,
                &code,
                SessionKind::Cli,
                None,
            )
            .await
        }
    };
    for _ in 0..5 {
        assert_eq!(
            attempt("wrong password here").await.err(),
            Some(CoreError::Unauthenticated)
        );
    }
    // Locked: even the right password is refused until the delay passes.
    assert!(matches!(
        attempt(PASSWORD).await,
        Err(CoreError::Throttled { .. })
    ));
    fixture.clock.advance(Duration::seconds(29));
    assert!(matches!(
        attempt(PASSWORD).await,
        Err(CoreError::Throttled { .. })
    ));
    fixture.clock.advance(Duration::minutes(15));
    assert!(attempt(PASSWORD).await.is_ok());
}

#[tokio::test]
async fn roles_follow_the_matrix_and_denials_are_audited() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let acme = fixture.organization(&owner, "acme").await;
    let auditor = fixture
        .member(&owner, &acme, "auditor@example.com", Role::Auditor)
        .await;

    assert_eq!(
        ok(fixture.hub.members(&auditor.context, acme.id).await).len(),
        2
    );
    assert!(matches!(
        fixture
            .hub
            .rename_organization(&auditor.context, acme.id, "New")
            .await,
        Err(CoreError::Forbidden { .. })
    ));
    assert!(matches!(
        fixture
            .hub
            .add_member(&auditor.context, acme.id, "x@example.com", Role::Owner)
            .await,
        Err(CoreError::Forbidden { .. })
    ));
    let visible = ok(fixture
        .hub
        .audit_records(&auditor.context, Some(acme.id), None, 100)
        .await);
    assert!(visible
        .iter()
        .any(|r| r.event.action == "organization.rename" && r.event.result == AuditResult::Denied));
    assert!(visible
        .iter()
        .all(|r| r.event.organization_id == Some(acme.id)));

    // Another organization is invisible to the auditor.
    let other = fixture.organization(&owner, "other").await;
    assert_eq!(
        fixture
            .hub
            .organization(&auditor.context, other.id)
            .await
            .err(),
        Some(CoreError::NotFound("organization"))
    );
}

#[tokio::test]
async fn the_last_owner_cannot_be_demoted_or_removed() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let acme = fixture.organization(&owner, "acme").await;
    let second = fixture
        .member(&owner, &acme, "second@example.com", Role::Owner)
        .await;
    let me = ok(fixture.hub.me(&owner.context).await).administrator.id;
    let other = ok(fixture.hub.me(&second.context).await).administrator.id;

    ok(fixture
        .hub
        .change_role(&owner.context, acme.id, other, Role::ReadOnly)
        .await);
    assert!(matches!(
        fixture
            .hub
            .change_role(&owner.context, acme.id, me, Role::Administrator)
            .await,
        Err(CoreError::Conflict(_))
    ));
    assert!(matches!(
        fixture.hub.remove_member(&owner.context, acme.id, me).await,
        Err(CoreError::Conflict(_))
    ));
}

#[tokio::test]
async fn a_recovery_code_works_once_and_forces_re_enrollment() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let code = owner.recovery_codes[0].clone();
    let anonymous = fixture.anonymous();
    let SignIn::MustReenroll(link) = ok(fixture
        .hub
        .sign_in(
            &anonymous,
            &owner.email,
            PASSWORD,
            &code,
            SessionKind::Cli,
            None,
        )
        .await)
    else {
        panic!("expected re-enrollment");
    };
    assert!(fixture
        .hub
        .sign_in(
            &anonymous,
            &owner.email,
            PASSWORD,
            &code,
            SessionKind::Cli,
            None
        )
        .await
        .is_err());
    let renewed = fixture.complete(&link.token, &owner.email).await;
    // The old authenticator no longer works; the new one does.
    let old = fixture.code(&owner.secret);
    assert!(fixture
        .hub
        .sign_in(
            &anonymous,
            &owner.email,
            PASSWORD,
            &old,
            SessionKind::Cli,
            None
        )
        .await
        .is_err());
    fixture.refresh(&renewed).await;
}

#[tokio::test]
async fn sessions_expire_on_inactivity_and_absolute_time_and_can_be_revoked() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let settings = InstanceSettings {
        session_inactivity_minutes: 15,
        session_absolute_hours: 1,
    };
    ok(fixture
        .hub
        .update_instance_settings(&owner.context, settings)
        .await);

    let open = |fixture: &Fixture| {
        let hub = fixture.hub.clone();
        let code = fixture.code(&owner.secret);
        let anonymous = fixture.anonymous();
        async move {
            match ok(hub
                .sign_in(
                    &anonymous,
                    "owner@example.com",
                    PASSWORD,
                    &code,
                    SessionKind::Console,
                    None,
                )
                .await)
            {
                SignIn::Session(issued) => issued,
                SignIn::MustReenroll(_) => panic!("unexpected"),
            }
        }
    };

    let idle = open(&fixture).await;
    fixture.clock.advance(Duration::minutes(16));
    assert!(fixture
        .hub
        .authenticate_session(&idle.token, None)
        .await
        .is_err());

    let busy = open(&fixture).await;
    for _ in 0..7 {
        fixture.clock.advance(Duration::minutes(10));
        let checked = fixture.hub.authenticate_session(&busy.token, None).await;
        if fixture.clock_elapsed_since(busy.session.created_at) >= Duration::hours(1) {
            assert!(checked.is_err());
        } else {
            assert!(checked.is_ok());
        }
    }

    let revoked = open(&fixture).await;
    let (context, session) = ok(fixture.hub.authenticate_session(&revoked.token, None).await);
    assert!(check_csrf(&session, revoked.csrf_token.as_deref()).is_ok());
    assert!(check_csrf(&session, Some("lvxc_forged")).is_err());
    assert!(check_csrf(&session, None).is_err());
    ok(fixture.hub.sign_out(&context).await);
    assert!(fixture
        .hub
        .authenticate_session(&revoked.token, None)
        .await
        .is_err());
}

#[tokio::test]
async fn api_tokens_act_with_the_creators_current_permissions() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let acme = fixture.organization(&owner, "acme").await;
    let admin = fixture
        .member(&owner, &acme, "admin@example.com", Role::Administrator)
        .await;
    let admin_id = ok(fixture.hub.me(&admin.context).await).administrator.id;

    assert!(fixture
        .hub
        .create_api_token(&admin.context, "ci", 91)
        .await
        .is_err());
    let (token, secret) = ok(fixture.hub.create_api_token(&admin.context, "ci", 30).await);
    assert!(secret.starts_with("lvxh_"));
    assert!(secret.starts_with(&token.display_prefix));

    let context = ok(fixture.hub.authenticate_api_token(&secret, None).await);
    assert_eq!(ok(fixture.hub.organizations(&context).await).len(), 1);
    // A token cannot mint tokens.
    assert!(fixture
        .hub
        .create_api_token(&context, "more", 1)
        .await
        .is_err());

    ok(fixture
        .hub
        .remove_member(&owner.context, acme.id, admin_id)
        .await);
    let context = ok(fixture.hub.authenticate_api_token(&secret, None).await);
    assert!(ok(fixture.hub.organizations(&context).await).is_empty());

    ok(fixture.hub.revoke_api_token(&admin.context, token.id).await);
    assert!(fixture
        .hub
        .authenticate_api_token(&secret, None)
        .await
        .is_err());

    let (_, short) = ok(fixture
        .hub
        .create_api_token(&admin.context, "short", 1)
        .await);
    fixture.clock.advance(Duration::days(1));
    assert!(fixture
        .hub
        .authenticate_api_token(&short, None)
        .await
        .is_err());
}

#[tokio::test]
async fn suspension_ends_sessions_and_owners_reset_second_factors() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let acme = fixture.organization(&owner, "acme").await;
    let member = fixture
        .member(&owner, &acme, "member@example.com", Role::Approver)
        .await;
    let id = ok(fixture.hub.me(&member.context).await).administrator.id;

    ok(fixture.hub.suspend_administrator(&owner.context, id).await);
    assert!(fixture
        .hub
        .me(&fixture.refresh_context(&member).await)
        .await
        .is_err());
    ok(fixture
        .hub
        .reactivate_administrator(&owner.context, id)
        .await);
    fixture.refresh(&member).await;

    let link = ok(fixture.hub.reset_second_factor(&owner.context, id).await);
    let renewed = fixture.complete(&link.token, &member.email).await;
    fixture.refresh(&renewed).await;

    // Nobody manages their own account through these actions.
    let me = ok(fixture.hub.me(&owner.context).await).administrator.id;
    assert!(matches!(
        fixture.hub.suspend_administrator(&owner.context, me).await,
        Err(CoreError::Forbidden { .. })
    ));
}

#[tokio::test]
async fn links_expire_and_the_host_recovers_the_instance_owner() {
    let fixture = Fixture::new().await;
    let (_, setup) = ok(fixture
        .hub
        .initialize(&Context::host(), "owner@example.com")
        .await);
    fixture.clock.advance(Duration::hours(25));
    assert_eq!(
        fixture
            .hub
            .inspect_link(&fixture.anonymous(), &setup.token)
            .await
            .err(),
        Some(CoreError::NotFound("link"))
    );
    let recovery = ok(fixture
        .hub
        .recover_owner(&Context::host(), "owner@example.com")
        .await);
    let owner = fixture.complete(&recovery.token, "owner@example.com").await;
    assert!(
        ok(fixture.hub.me(&owner.context).await)
            .administrator
            .is_instance_owner
    );
    assert!(fixture
        .hub
        .recover_owner(&Context::host(), "someone@example.com")
        .await
        .is_err());
}

#[tokio::test]
async fn the_audit_chain_is_intact_after_a_working_session() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let acme = fixture.organization(&owner, "acme").await;
    fixture
        .member(&owner, &acme, "auditor@example.com", Role::Auditor)
        .await;
    let report = ok(fixture.hub.verify_audit(&Context::host()).await);
    assert!(report.intact);
    assert!(report.records >= 5);
    let page = ok(fixture
        .hub
        .audit_records(&owner.context, None, None, 2)
        .await);
    assert_eq!(page.len(), 2);
    assert!(page[0].seq > page[1].seq);
    let next = ok(fixture
        .hub
        .audit_records(&owner.context, None, Some(page[1].seq), 500)
        .await);
    assert_eq!(next.len() as u64 + 2, report.records);
}

#[tokio::test]
async fn completing_a_link_needs_a_matching_code_and_a_good_password() {
    let fixture = Fixture::new().await;
    let (_, link) = ok(fixture
        .hub
        .initialize(&Context::host(), "owner@example.com")
        .await);
    let anonymous = fixture.anonymous();
    let complete = |password: &'static str, code: &'static str| {
        fixture.hub.complete_link(
            &anonymous,
            LinkCompletion {
                token: &link.token,
                display_name: Some("Owner"),
                password,
                code,
            },
            SessionKind::Console,
            None,
        )
    };
    assert!(matches!(
        complete(PASSWORD, "123456").await,
        Err(CoreError::Conflict(_))
    ));
    ok(fixture.hub.begin_enrollment(&anonymous, &link.token).await);
    assert!(matches!(
        complete(PASSWORD, "123456").await,
        Err(CoreError::Invalid { field: "code", .. })
    ));
    assert!(fixture
        .hub
        .inspect_link(&anonymous, &link.token)
        .await
        .is_ok());
}

#[tokio::test]
async fn concurrent_failures_are_all_counted() {
    let fixture = Fixture::new().await;
    let owner = fixture.owner().await;
    let anonymous = fixture.anonymous();
    let code = fixture.current_code(&owner.secret);
    let wrong = || {
        fixture.hub.sign_in(
            &anonymous,
            "owner@example.com",
            "wrong password here",
            &code,
            SessionKind::Cli,
            None,
        )
    };
    let _ = tokio::join!(wrong(), wrong(), wrong(), wrong(), wrong());
    let locked = fixture
        .hub
        .sign_in(
            &anonymous,
            "owner@example.com",
            PASSWORD,
            &code,
            SessionKind::Cli,
            None,
        )
        .await;
    assert!(
        matches!(locked, Err(CoreError::Throttled { .. })),
        "{locked:?}"
    );
}
