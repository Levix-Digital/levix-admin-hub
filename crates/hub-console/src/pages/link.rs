//! A setup, invitation or recovery link: set the password and the authenticator, then sign in.

use leptos::prelude::*;
use serde_json::{json, Value};

use crate::api::{self, ApiError};
use crate::i18n::{t, t_with};
use crate::pages::{finish_sign_in, RecoveryCodes};
use crate::ui::{qr_data_url, run, session, text, when, ErrorMessage, Title};

fn mismatch() -> ApiError {
    ApiError {
        status: 0,
        code: "password_mismatch".to_owned(),
        message: String::new(),
    }
}

#[component]
pub fn LinkPage(token: String) -> impl IntoView {
    let session = session();
    let token = StoredValue::new(token);
    let info = RwSignal::new(None::<Value>);
    let enrollment = RwSignal::new(None::<Value>);
    let codes = RwSignal::new(None::<(Vec<String>, Value)>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);

    let display_name = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm = RwSignal::new(String::new());
    let code = RwSignal::new(String::new());

    {
        let token = token.get_value();
        run(
            busy,
            error,
            async move {
                api::public_post("/api/v1/auth/links/inspect", &json!({"token": token})).await
            },
            move |value| info.set(Some(value)),
        );
    }

    let enroll = {
        move |_| {
            let token = token.get_value();
            run(
                busy,
                error,
                async move {
                    api::public_post("/api/v1/auth/links/enrollment", &json!({"token": token}))
                        .await
                },
                move |value| enrollment.set(Some(value)),
            );
        }
    };

    let submit = {
        move |ev: leptos::ev::SubmitEvent| {
            let token = token.get_value();
            ev.prevent_default();
            if password.get_untracked() != confirm.get_untracked() {
                error.set(Some(mismatch()));
                return;
            }
            let name = display_name.get_untracked();
            let body = json!({
                "token": token,
                "password": password.get_untracked(),
                "code": code.get_untracked(),
                "display_name": if name.trim().is_empty() { Value::Null } else { json!(name) },
                "client": "console",
            });
            run(
                busy,
                error,
                async move { api::public_post("/api/v1/auth/links/complete", &body).await },
                move |answer: Value| {
                    password.set(String::new());
                    confirm.set(String::new());
                    let issued: Vec<String> = answer
                        .get("recovery_codes")
                        .and_then(Value::as_array)
                        .map(|c| {
                            c.iter()
                                .filter_map(|v| v.as_str().map(str::to_owned))
                                .collect()
                        })
                        .unwrap_or_default();
                    if issued.is_empty() {
                        finish_sign_in(session, &answer);
                    } else {
                        codes.set(Some((issued, answer)));
                    }
                },
            );
        }
    };

    let purpose = move || {
        info.get()
            .map(|i| {
                t(&format!(
                    "purpose-{}",
                    text(&i, "/purpose").replace('_', "-")
                ))
            })
            .unwrap_or_default()
    };

    view! {
        <section class="narrow">
            <Title key="link-title" />
            {move || info.get().map(|i| view! {
                <p>{move || t_with("link-intro", &[
                    ("purpose", purpose()),
                    ("email", text(&i, "/email")),
                    ("expires", when(&text(&i, "/expires_at"))),
                ])}</p>
            })}
            <Show when=move || codes.get().is_none() fallback=move || {
                let (issued, answer) = codes.get().unwrap_or_default();
                view! {
                    <RecoveryCodes codes=issued />
                    <button type="button" on:click=move |_| finish_sign_in(session, &answer)>
                        {move || t("codes-continue")}
                    </button>
                }
            }>
                <Show when=move || info.get().is_some()>
                    <Show when=move || enrollment.get().is_some() fallback={
                        move || view! {
                            <p>{move || t("enroll-intro")}</p>
                            <button type="button" on:click=enroll disabled=move || busy.get()>
                                {move || t("enroll-start")}
                            </button>
                        }
                    }>
                        {move || enrollment.get().map(|e| {
                            let uri = text(&e, "/uri");
                            let secret = text(&e, "/secret");
                            view! {
                                <h2>{move || t("enroll-title")}</h2>
                                <p>{move || t("enroll-scan")}</p>
                                {qr_data_url(&uri).map(|src| view! {
                                    <img class="qr" src=src alt=move || t("enroll-qr-alt") width="200" height="200" />
                                })}
                                <p>{move || t("enroll-manual")}</p>
                                <p><code class="secret" data-testid="totp-secret">{secret}</code></p>
                            }
                        })}
                        <form on:submit=submit>
                            <label for="link-name">{move || t("field-display-name")}</label>
                            <input id="link-name" autocomplete="name" bind:value=display_name />
                            <label for="link-password">{move || t("field-new-password")}</label>
                            <input id="link-password" type="password" autocomplete="new-password" required
                                minlength="12" aria-describedby="link-password-help" bind:value=password />
                            <p id="link-password-help" class="help">{move || t("field-password-help")}</p>
                            <label for="link-confirm">{move || t("field-confirm-password")}</label>
                            <input id="link-confirm" type="password" autocomplete="new-password" required bind:value=confirm />
                            <label for="link-code">{move || t("field-code")}</label>
                            <input id="link-code" inputmode="numeric" autocomplete="one-time-code" required bind:value=code />
                            <button type="submit" disabled=move || busy.get()>{move || t("link-submit")}</button>
                        </form>
                    </Show>
                </Show>
            </Show>
            <ErrorMessage error=error />
        </section>
    }
}
