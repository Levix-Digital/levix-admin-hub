//! `hub-console`: the web console of the Admin Hub, compiled to WebAssembly and embedded in
//! the server binary. It calls only the public API, with the session cookie.

mod api;
mod app;
mod i18n;
mod idle;
mod pages;
mod route;
mod ui;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::i18n::{errors, keys, LANGUAGES};

    /// Every language parses and has exactly the keys of English.
    #[test]
    fn translations_are_complete() {
        let english: BTreeSet<String> = keys("en").into_iter().collect();
        assert!(english.len() > 100, "{}", english.len());
        for (language, _) in LANGUAGES {
            assert_eq!(errors(language), 0, "{language} has syntax errors");
            let other: BTreeSet<String> = keys(language).into_iter().collect();
            let missing: Vec<_> = english.difference(&other).collect();
            let extra: Vec<_> = other.difference(&english).collect();
            assert!(
                missing.is_empty() && extra.is_empty(),
                "{language}: missing {missing:?}, extra {extra:?}"
            );
        }
    }

    #[test]
    fn languages_match_browser_tags() {
        use crate::i18n::supported;
        assert_eq!(supported("pt-BR"), Some("pt-BR"));
        assert_eq!(supported("pt-PT"), Some("pt-BR"));
        assert_eq!(supported("fr"), Some("fr-CA"));
        assert_eq!(supported("en-GB"), Some("en"));
        assert_eq!(supported("de"), None);
    }
}
