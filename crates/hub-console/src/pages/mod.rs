//! The pages of the console.

pub mod audit;
pub mod instance;
pub mod link;
pub mod login;
pub mod organization;
pub mod organizations;
pub mod profile;
pub mod sessions;
pub mod tokens;

use leptos::prelude::*;
use serde_json::Value;

use crate::api;
use crate::i18n::t;
use crate::route::{self, Route};
use crate::ui::{Session, Title};

/// Shown while the Hub has not been initialized on its host.
#[component]
pub fn NotInitialized() -> impl IntoView {
    view! {
        <section>
            <Title key="not-initialized-title" />
            <p>{move || t("not-initialized-text")}</p>
            <pre><code>"levix-hub init --owner-email you@example.com"</code></pre>
            <p>{move || t("not-initialized-next")}</p>
        </section>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <section>
            <Title key="not-found-title" />
            <p><a href="#/">{move || t("not-found-home")}</a></p>
        </section>
    }
}

/// Ends a sign-in (login or link): keeps the CSRF token, loads the account, opens the
/// organizations. A reset authenticator goes to its re-enrollment link instead.
pub fn finish_sign_in(session: Session, body: &Value) {
    match body.get("status").and_then(Value::as_str) {
        Some("must_reenroll") => {
            if let Some(token) = body.pointer("/link/token").and_then(Value::as_str) {
                session.notice.set(Some("notice-reenroll".to_owned()));
                route::go(&Route::Link(token.to_owned()));
            }
        }
        _ => {
            api::set_csrf(body.get("csrf_token").and_then(Value::as_str));
            leptos::task::spawn_local(async move {
                if let Ok(me) = api::get("/api/v1/me").await {
                    session.notice.set(None);
                    session.me.set(Some(me));
                    route::go(&Route::Organizations);
                }
            });
        }
    }
}

/// One-time codes shown after they are issued.
#[component]
pub fn RecoveryCodes(codes: Vec<String>) -> impl IntoView {
    view! {
        <section class="codes" aria-labelledby="codes-title">
            <h2 id="codes-title">{move || t("codes-title")}</h2>
            <p>{move || t("codes-text")}</p>
            <ul class="code-list">
                {codes.into_iter().map(|code| view! { <li><code>{code}</code></li> }).collect_view()}
            </ul>
        </section>
    }
}
