//! Translations in Fluent (`en`, `fr-CA`, `pt-BR`), chosen from the profile, else from the
//! browser's languages, else English.

use std::cell::RefCell;
use std::collections::HashMap;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use leptos::prelude::*;
use unic_langid::LanguageIdentifier;

/// The languages of the console, in the order of the language menu.
pub const LANGUAGES: [(&str, &str); 3] = [
    ("en", "English"),
    ("fr-CA", "Français (Canada)"),
    ("pt-BR", "Português (Brasil)"),
];

const SOURCES: [(&str, &str); 3] = [
    ("en", include_str!("../locales/en.ftl")),
    ("fr-CA", include_str!("../locales/fr-CA.ftl")),
    ("pt-BR", include_str!("../locales/pt-BR.ftl")),
];

thread_local! {
    static BUNDLES: RefCell<HashMap<&'static str, FluentBundle<FluentResource>>> =
        RefCell::new(HashMap::new());
}

fn with_bundle<R>(language: &str, f: impl FnOnce(&FluentBundle<FluentResource>) -> R) -> R {
    let language = supported(language).unwrap_or("en");
    BUNDLES.with(|bundles| {
        let mut bundles = bundles.borrow_mut();
        let bundle = bundles.entry(language).or_insert_with(|| build(language));
        f(bundle)
    })
}

fn build(language: &'static str) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = language.parse().unwrap_or_default();
    let mut bundle = FluentBundle::new(vec![id]);
    bundle.set_use_isolating(false);
    let source = SOURCES
        .iter()
        .find(|(l, _)| *l == language)
        .map(|(_, s)| *s)
        .unwrap_or_default();
    let resource = FluentResource::try_new(source.to_owned()).unwrap_or_else(|(r, _)| r);
    let _ignored = bundle.add_resource(resource);
    bundle
}

/// The supported language matching a browser or profile tag (`pt`, `pt-PT` → `pt-BR`).
pub fn supported(tag: &str) -> Option<&'static str> {
    let tag = tag.trim();
    LANGUAGES
        .iter()
        .map(|(l, _)| *l)
        .find(|l| l.eq_ignore_ascii_case(tag))
        .or_else(|| {
            let primary = tag.split(['-', '_']).next().unwrap_or_default();
            LANGUAGES.iter().map(|(l, _)| *l).find(|l| {
                l.split('-')
                    .next()
                    .is_some_and(|p| p.eq_ignore_ascii_case(primary))
            })
        })
}

/// The first supported language among the browser's preferences.
pub fn from_browser() -> &'static str {
    let navigator = web_sys::window().map(|w| w.navigator());
    let languages: Vec<String> = navigator
        .map(|n| {
            n.languages()
                .iter()
                .filter_map(|v| v.as_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    languages.iter().find_map(|l| supported(l)).unwrap_or("en")
}

thread_local! {
    /// The current language, shared by the whole console. A global rather than a context,
    /// so event handlers and finished requests can translate too.
    static LANGUAGE: RwSignal<String> = RwSignal::new("en".to_owned());
}

pub fn language() -> RwSignal<String> {
    LANGUAGE.with(|l| *l)
}

fn format(language: &str, key: &str, args: Option<&FluentArgs>) -> String {
    with_bundle(language, |bundle| {
        let Some(pattern) = bundle.get_message(key).and_then(|m| m.value()) else {
            return key.to_owned();
        };
        let mut errors = Vec::new();
        bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned()
    })
}

/// The text of `key` in the current language; reactive when called inside a view closure.
pub fn t(key: &str) -> String {
    format(&language().get(), key, None)
}

/// The text of `key` with arguments.
pub fn t_with(key: &str, args: &[(&str, String)]) -> String {
    let mut fluent = FluentArgs::new();
    for (name, value) in args {
        fluent.set(*name, FluentValue::from(value.clone()));
    }
    format(&language().get(), key, Some(&fluent))
}

/// Every key of a language, for the completeness test.
#[cfg(test)]
pub fn keys(language: &str) -> Vec<String> {
    let source = SOURCES
        .iter()
        .find(|(l, _)| *l == language)
        .map(|(_, s)| *s)
        .unwrap_or_default();
    let resource = FluentResource::try_new(source.to_owned()).unwrap_or_else(|(r, _)| r);
    resource
        .entries()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(m) => Some(m.id.name.to_owned()),
            _ => None,
        })
        .collect()
}

/// Parse errors of a language file, for the test.
#[cfg(test)]
pub fn errors(language: &str) -> usize {
    let source = SOURCES
        .iter()
        .find(|(l, _)| *l == language)
        .map(|(_, s)| *s)
        .unwrap_or_default();
    match FluentResource::try_new(source.to_owned()) {
        Ok(_) => 0,
        Err((_, errors)) => errors.len(),
    }
}
