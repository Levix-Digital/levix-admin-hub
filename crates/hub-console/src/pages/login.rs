//! Sign-in with password and authenticator code.

use leptos::prelude::*;
use serde_json::json;

use crate::api::{self, ApiError};
use crate::i18n::t;
use crate::pages::finish_sign_in;
use crate::ui::{run, session, ErrorMessage, Title};

#[component]
pub fn Login() -> impl IntoView {
    let session = session();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let code = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = json!({
            "email": email.get_untracked(),
            "password": password.get_untracked(),
            "code": code.get_untracked(),
            "client": "console",
        });
        run(
            busy,
            error,
            async move { api::public_post("/api/v1/auth/login", &body).await },
            move |answer| {
                password.set(String::new());
                code.set(String::new());
                finish_sign_in(session, &answer);
            },
        );
    };

    view! {
        <section class="narrow">
            <Title key="login-title" />
            <form on:submit=submit>
                <label for="login-email">{move || t("field-email")}</label>
                <input id="login-email" type="email" autocomplete="username" required bind:value=email />
                <label for="login-password">{move || t("field-password")}</label>
                <input id="login-password" type="password" autocomplete="current-password" required bind:value=password />
                <label for="login-code">{move || t("field-code")}</label>
                <input id="login-code" inputmode="numeric" autocomplete="one-time-code" required bind:value=code
                    aria-describedby="login-code-help" />
                <p id="login-code-help" class="help">{move || t("field-code-help")}</p>
                <ErrorMessage error=error />
                <button type="submit" disabled=move || busy.get()>{move || t("login-submit")}</button>
            </form>
        </section>
    }
}
