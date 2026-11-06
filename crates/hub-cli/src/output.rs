//! What a command prints: the Hub's JSON as received in `--output json`, short text otherwise.

use serde_json::Value;

/// The result of one command.
#[derive(Debug)]
pub struct Report {
    pub json: Value,
    pub human: String,
    pub exit: u8,
}

impl Report {
    pub fn new(json: Value, human: String) -> Self {
        Self {
            json,
            human,
            exit: crate::error::exit::OK,
        }
    }

    pub fn with_exit(mut self, exit: u8) -> Self {
        self.exit = exit;
        self
    }
}

/// A field as text: strings bare, `null` as `-`, the rest as compact JSON.
pub fn text(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "-".to_owned(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// `key: value` lines for the named fields of an object.
pub fn fields(value: &Value, keys: &[&str]) -> String {
    let width = keys.iter().map(|k| k.len()).max().unwrap_or(0);
    keys.iter()
        .map(|key| format!("{key:width$}  {}", text(value.get(*key))))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A table of the named columns of `{"items": [...]}` or of a bare array.
pub fn table(value: &Value, columns: &[&str]) -> String {
    let empty = Vec::new();
    let rows = value
        .get("items")
        .and_then(Value::as_array)
        .or_else(|| value.as_array())
        .unwrap_or(&empty);
    if rows.is_empty() {
        return "(none)".to_owned();
    }
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| columns.iter().map(|c| text(row.get(*c))).collect())
        .collect();
    let widths: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            cells
                .iter()
                .map(|r| r[i].chars().count())
                .chain(std::iter::once(c.len()))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |values: Vec<String>| {
        values
            .iter()
            .zip(&widths)
            .map(|(v, w)| format!("{v:w$}"))
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_owned()
    };
    let mut lines = vec![line(columns.iter().map(|c| c.to_uppercase()).collect())];
    lines.extend(cells.into_iter().map(line));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tables_align_and_mark_missing_values() {
        let value =
            json!({"items": [{"slug": "acme", "name": "Acme"}, {"slug": "b", "name": null}]});
        assert_eq!(
            table(&value, &["slug", "name"]),
            "SLUG  NAME\nacme  Acme\nb     -"
        );
        assert_eq!(table(&json!({"items": []}), &["slug"]), "(none)");
    }
}
