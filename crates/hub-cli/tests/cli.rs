//! The CLI against a real Hub over TLS (T031): sign-in, administration, roles, audit, tokens,
//! trust of the certificate and exit codes.

mod common;

use common::{run_in, Hub, User};

const PASSWORD: &str = "correct horse battery staple\n";

#[test]
fn administers_a_hub_end_to_end() {
    let hub = Hub::start();
    let fingerprint = hub.fingerprint.as_str();
    let mut owner = User::new();

    // The setup link names the Hub, so no --url is needed to claim it.
    let inspected = owner
        .run(
            &[
                "--fingerprint",
                fingerprint,
                "link",
                "inspect",
                &hub.setup_link,
            ],
            None,
            &[],
        )
        .success();
    assert!(
        inspected.stdout.contains("owner@example.com"),
        "{}",
        inspected.stdout
    );
    let claimed = owner.complete(&hub, &hub.setup_link, "Owner").success();
    assert!(
        claimed.stdout.contains("Recovery codes"),
        "{}",
        claimed.stdout
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = common::ok(std::fs::metadata(owner.session_file()))
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    let saved = common::ok(std::fs::read_to_string(owner.session_file()));
    assert!(saved.contains(&hub.url) && saved.contains("lvx"), "{saved}");

    let status = owner.run(&["status"], None, &[]).success();
    assert!(
        status.stdout.contains("owner@example.com"),
        "{}",
        status.stdout
    );

    // Sign out and in again with the saved address.
    owner.run(&["logout"], None, &[]).success();
    assert!(!owner.session_file().exists());
    assert_eq!(
        owner.run(&["me", "show"], None, &[]).code,
        2,
        "no address any more"
    );
    let code = owner.next_code();
    let login = owner
        .run(
            &[
                "--url",
                &hub.url,
                "--fingerprint",
                fingerprint,
                "--output",
                "json",
                "login",
                "--email",
                "owner@example.com",
                "--password-stdin",
                "--code",
                &code,
            ],
            Some(PASSWORD),
            &[],
        )
        .success()
        .json();
    assert_eq!(login["status"], "signed_in");
    assert_eq!(
        login["token"], "(saved)",
        "the session token is not printed"
    );

    // Organizations and members.
    owner
        .run(
            &["org", "create", "--slug", "acme", "--name", "Acme"],
            None,
            &[],
        )
        .success();
    let list = owner.run(&["org", "list"], None, &[]).success();
    assert!(list.stdout.contains("acme"), "{}", list.stdout);
    let added = owner
        .run(
            &[
                "--output",
                "json",
                "member",
                "add",
                "acme",
                "--email",
                "auditor@example.com",
                "--role",
                "auditor",
            ],
            None,
            &[],
        )
        .success()
        .json();
    let invitation = added["invitation"]["token"].as_str().unwrap_or_default();
    assert!(invitation.starts_with("lvxl_"), "{added}");

    // The auditor accepts the invitation and may read but not change.
    let mut auditor = User::new();
    let link = format!("{}/#/link/{invitation}", hub.url);
    auditor.complete(&hub, &link, "Auditor").success();
    let members = auditor
        .run(&["member", "list", "acme"], None, &[])
        .success();
    assert!(
        members.stdout.contains("auditor@example.com"),
        "{}",
        members.stdout
    );
    let denied = auditor.run(
        &[
            "--output", "json", "org", "rename", "acme", "--name", "Other",
        ],
        None,
        &[],
    );
    assert_eq!(denied.code, 4, "{}", denied.stderr);
    assert!(denied.stderr.contains("\"code\""), "{}", denied.stderr);

    owner
        .run(
            &[
                "member",
                "set-role",
                "acme",
                "auditor@example.com",
                "--role",
                "read_only",
            ],
            None,
            &[],
        )
        .success();
    let members = owner
        .run(&["--output", "json", "member", "list", "acme"], None, &[])
        .success()
        .json();
    assert!(
        members["items"]
            .as_array()
            .is_some_and(|items| items.iter().any(|m| m["role"] == "read_only")),
        "{members}"
    );
    let missing = owner.run(&["org", "show", "nowhere"], None, &[]);
    assert_eq!(missing.code, 5, "{}", missing.stderr);
    let invalid = owner.run(
        &["org", "create", "--slug", "Not A Slug", "--name", "X"],
        None,
        &[],
    );
    assert_eq!(invalid.code, 7, "{}", invalid.stderr);

    // Audit: every change above is recorded and the chain verifies.
    let audit = owner
        .run(
            &["--output", "json", "audit", "list", "--all", "--limit", "2"],
            None,
            &[],
        )
        .success()
        .json();
    let actions: Vec<&str> = audit["items"]
        .as_array()
        .map(|items| items.iter().filter_map(|r| r["action"].as_str()).collect())
        .unwrap_or_default();
    assert!(actions.contains(&"organization.create"), "{actions:?}");
    assert!(actions.len() > 2, "--all follows the pages: {actions:?}");
    owner.run(&["audit", "verify"], None, &[]).success();

    // A personal API token works without any saved session.
    let created = owner
        .run(
            &[
                "--output", "json", "token", "create", "--name", "ci", "--days", "1",
            ],
            None,
            &[],
        )
        .success()
        .json();
    let secret = created["secret"].as_str().unwrap_or_default();
    let elsewhere = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let anonymous = run_in(
        elsewhere.path(),
        &[
            "--url",
            &hub.url,
            "--fingerprint",
            fingerprint,
            "org",
            "list",
        ],
        None,
        &[],
    );
    assert_eq!(anonymous.code, 3, "not signed in: {}", anonymous.stderr);
    let with_token = run_in(
        elsewhere.path(),
        &[
            "--url",
            &hub.url,
            "--fingerprint",
            fingerprint,
            "org",
            "list",
        ],
        None,
        &[("LEVIX_HUB_TOKEN", secret)],
    )
    .success();
    assert!(with_token.stdout.contains("acme"), "{}", with_token.stdout);
}

#[test]
fn trusts_only_the_expected_certificate() {
    let hub = Hub::start();
    let user = User::new();
    let health = |args: &[&str]| user.run(args, None, &[]);

    let pinned = health(&[
        "--url",
        &hub.url,
        "--fingerprint",
        &hub.fingerprint,
        "health",
    ])
    .success();
    assert!(pinned.stdout.contains("ready"), "{}", pinned.stdout);

    let certificate = hub.certificate.to_string_lossy();
    health(&["--url", &hub.url, "--ca-file", &certificate, "health"]).success();

    let unknown = health(&["--url", &hub.url, "health"]);
    assert_eq!(
        unknown.code, 9,
        "an unknown certificate is refused: {}",
        unknown.stderr
    );

    let other = "00".repeat(32);
    let wrong = health(&["--url", &hub.url, "--fingerprint", &other, "health"]);
    assert_eq!(wrong.code, 9, "{}", wrong.stderr);
    assert!(wrong.stderr.contains("fingerprint"), "{}", wrong.stderr);

    let usage = health(&["--url", "hub.example", "health"]);
    assert_eq!(usage.code, 2, "{}", usage.stderr);
}
