//! SC-004: every operation of the committed API contract has a CLI command, and every
//! command named in the table exists.

use std::collections::BTreeSet;

use clap::CommandFactory;
use hub_cli::cli::Cli;
use hub_cli::operations::OPERATIONS;
use serde_json::Value;

#[test]
fn every_api_operation_has_a_command() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/contracts/api/openapi.json"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let document: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}"));
    let mut contract = BTreeSet::new();
    for (path, operations) in document["paths"].as_object().into_iter().flatten() {
        for method in operations.as_object().into_iter().flatten().map(|(m, _)| m) {
            contract.insert((method.clone(), path.clone()));
        }
    }
    let table: BTreeSet<(String, String)> = OPERATIONS
        .iter()
        .map(|(method, path, _)| ((*method).to_owned(), (*path).to_owned()))
        .collect();
    assert_eq!(
        table.len(),
        OPERATIONS.len(),
        "an operation is listed twice"
    );
    assert_eq!(contract, table, "the CLI table and the contract differ");
}

#[test]
fn every_listed_command_exists() {
    let root = Cli::command();
    for (_, _, command) in OPERATIONS {
        let mut current = &root;
        for word in command.split(' ') {
            current = current
                .find_subcommand(word)
                .unwrap_or_else(|| panic!("no command `{command}`"));
        }
    }
}
