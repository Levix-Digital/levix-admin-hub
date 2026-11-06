//! The committed OpenAPI document equals the one generated from the code (ADR-0006).
//! Regenerate it with `LEVIX_UPDATE_CONTRACT=1 cargo test -p hub-api --test contract`.

use std::path::PathBuf;

fn committed() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/api/openapi.json")
}

#[test]
fn committed_openapi_matches_the_code() {
    let generated = hub_api::openapi_json();
    let path = committed();
    if std::env::var_os("LEVIX_UPDATE_CONTRACT").is_some() {
        std::fs::write(&path, &generated).unwrap_or_else(|e| panic!("{e}"));
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        on_disk == generated,
        "docs/contracts/api/openapi.json is out of date; run \
         LEVIX_UPDATE_CONTRACT=1 cargo test -p hub-api --test contract and commit it"
    );
}

#[test]
fn every_operation_is_documented_with_its_errors() {
    let document: serde_json::Value =
        serde_json::from_str(&hub_api::openapi_json()).unwrap_or_default();
    assert_eq!(document["info"]["version"], hub_api::CONTRACT_VERSION);
    let paths = document["paths"].as_object().cloned().unwrap_or_default();
    let operations: Vec<_> = paths
        .values()
        .filter_map(|item| item.as_object())
        .flat_map(|item| item.values().cloned())
        .collect();
    assert_eq!(
        operations.len(),
        34,
        "an endpoint is missing from the document"
    );
    for operation in &operations {
        let responses = operation["responses"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        assert!(!responses.is_empty(), "{operation}");
        assert!(operation["tags"].is_array(), "{operation}");
    }
}

fn channel_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/channel/v1")
}

fn read_json(path: &std::path::Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn validator(name: &str) -> jsonschema::Validator {
    let schema = read_json(&channel_dir().join(format!("schemas/{name}.schema.json")));
    jsonschema::validator_for(&schema).unwrap_or_else(|e| panic!("{name}: {e}"))
}

const CHANNEL_TYPES: [&str; 5] = ["hello", "challenge", "authenticate", "welcome", "error"];
const FROM_HUB: [&str; 3] = ["challenge", "welcome", "error"];

/// Checks one envelope: the envelope schema, then the payload schema of its type.
fn check_message(message: &serde_json::Value) -> Result<(), String> {
    let errors: Vec<String> = validator("envelope")
        .iter_errors(message)
        .map(|e| e.to_string())
        .collect();
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let kind = message["type"].as_str().unwrap_or_default();
    let errors: Vec<String> = validator(kind)
        .iter_errors(&message["payload"])
        .map(|e| e.to_string())
        .collect();
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    if FROM_HUB.contains(&kind) && message.get("sig").is_none() {
        return Err(format!("{kind} comes from the Hub and must be signed"));
    }
    Ok(())
}

#[test]
fn channel_examples_follow_their_schemas() {
    for kind in CHANNEL_TYPES {
        let example = read_json(&channel_dir().join(format!("examples/{kind}.json")));
        assert_eq!(example["type"], kind);
        if let Err(error) = check_message(&example) {
            panic!("example {kind}: {error}");
        }
    }
}

#[test]
fn channel_schemas_refuse_unknown_members_and_unsigned_hub_messages() {
    let mut hello = read_json(&channel_dir().join("examples/hello.json"));
    hello["payload"]["document_text"] = serde_json::json!("content must never travel");
    assert!(check_message(&hello).is_err());

    let mut envelope = read_json(&channel_dir().join("examples/welcome.json"));
    envelope["extra"] = serde_json::json!(1);
    assert!(check_message(&envelope).is_err());

    let mut unsigned = read_json(&channel_dir().join("examples/challenge.json"));
    if let Some(object) = unsigned.as_object_mut() {
        object.remove("sig");
    }
    assert!(check_message(&unsigned).is_err());

    let mut wrong_major = read_json(&channel_dir().join("examples/hello.json"));
    wrong_major["v"] = serde_json::json!(2);
    assert!(check_message(&wrong_major).is_err());
}

#[test]
fn asyncapi_lists_every_message_with_its_schema_and_example() {
    let document = read_json(&channel_dir().join("asyncapi.json"));
    assert_eq!(document["asyncapi"], "3.0.0");
    let messages = &document["channels"]["device"]["messages"];
    for kind in CHANNEL_TYPES {
        let message = &messages[kind];
        assert_eq!(
            message["x-payload-schema"]["$ref"],
            format!("schemas/{kind}.schema.json")
        );
        let example = &message["examples"][0]["payload"];
        assert_eq!(
            example,
            &read_json(&channel_dir().join(format!("examples/{kind}.json"))),
            "the example in asyncapi.json differs from examples/{kind}.json"
        );
    }
}
