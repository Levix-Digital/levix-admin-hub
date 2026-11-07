//! The administrator's own account: display name, password and recovery codes.

use leptos::prelude::*;
use serde_json::{json, Value};

use crate::api::{self, ApiError};
use crate::i18n::t;
use crate::pages::RecoveryCodes;
use crate::ui::{run, session, text, ErrorMessage, Title};

#[component]
pub fn Profile() -> impl IntoView {
    let session = session();
    let busy = RwSignal::new(false);

    let display_name = RwSignal::new(
        session
            .me
            .get_untracked()
            .map(|me| text(&me, "/display_name"))
            .unwrap_or_default(),
    );
    let profile_error = RwSignal::new(None::<ApiError>);
    let saved = RwSignal::new(false);
    let save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        saved.set(false);
        let body = json!({"display_name": display_name.get_untracked()});
        run(
            busy,
            profile_error,
            async move { api::patch("/api/v1/me", &body).await },
            move |me| {
                session.me.set(Some(me));
                saved.set(true);
            },
        );
    };

    let current = RwSignal::new(String::new());
    let new = RwSignal::new(String::new());
    let confirm = RwSignal::new(String::new());
    let password_code = RwSignal::new(String::new());
    let password_error = RwSignal::new(None::<ApiError>);
    let password_done = RwSignal::new(false);
    let change_password = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        password_done.set(false);
        if new.get_untracked() != confirm.get_untracked() {
            password_error.set(Some(ApiError {
                status: 0,
                code: "password_mismatch".to_owned(),
                message: String::new(),
            }));
            return;
        }
        let body = json!({
            "current_password": current.get_untracked(),
            "new_password": new.get_untracked(),
            "code": password_code.get_untracked(),
        });
        run(
            busy,
            password_error,
            async move { api::post_reauthenticated("/api/v1/me/password", &body).await },
            move |_| {
                for field in [current, new, confirm, password_code] {
                    field.set(String::new());
                }
                password_done.set(true);
            },
        );
    };

    let codes_password = RwSignal::new(String::new());
    let codes_code = RwSignal::new(String::new());
    let codes_error = RwSignal::new(None::<ApiError>);
    let codes = RwSignal::new(None::<Vec<String>>);
    let regenerate = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body =
            json!({"password": codes_password.get_untracked(), "code": codes_code.get_untracked()});
        run(
            busy,
            codes_error,
            async move { api::post_reauthenticated("/api/v1/me/recovery-codes", &body).await },
            move |v: Value| {
                codes_password.set(String::new());
                codes_code.set(String::new());
                codes.set(Some(
                    v.get("recovery_codes")
                        .and_then(Value::as_array)
                        .map(|c| {
                            c.iter()
                                .filter_map(|x| x.as_str().map(str::to_owned))
                                .collect()
                        })
                        .unwrap_or_default(),
                ));
            },
        );
    };

    view! {
        <section>
            <Title key="profile-title" />
            {move || session.me.get().map(|me| view! {
                <dl>
                    <dt>{move || t("field-email")}</dt><dd>{text(&me, "/email")}</dd>
                </dl>
            })}

            <h2>{move || t("profile-details")}</h2>
            <form on:submit=save>
                <label for="profile-name">{move || t("field-display-name")}</label>
                <input id="profile-name" autocomplete="name" required bind:value=display_name />
                <ErrorMessage error=profile_error />
                <Show when=move || saved.get()><p role="status">{move || t("saved")}</p></Show>
                <button type="submit" disabled=move || busy.get()>{move || t("save")}</button>
            </form>

            <h2>{move || t("password-title")}</h2>
            <p class="help">{move || t("password-text")}</p>
            <form on:submit=change_password>
                <label for="pw-current">{move || t("field-current-password")}</label>
                <input id="pw-current" type="password" autocomplete="current-password" required bind:value=current />
                <label for="pw-new">{move || t("field-new-password")}</label>
                <input id="pw-new" type="password" autocomplete="new-password" required minlength="12" bind:value=new />
                <label for="pw-confirm">{move || t("field-confirm-password")}</label>
                <input id="pw-confirm" type="password" autocomplete="new-password" required bind:value=confirm />
                <label for="pw-code">{move || t("field-code")}</label>
                <input id="pw-code" inputmode="numeric" autocomplete="one-time-code" required bind:value=password_code />
                <ErrorMessage error=password_error />
                <Show when=move || password_done.get()><p role="status">{move || t("password-changed")}</p></Show>
                <button type="submit" disabled=move || busy.get()>{move || t("password-submit")}</button>
            </form>

            <h2>{move || t("codes-regenerate-title")}</h2>
            <p class="help">{move || t("codes-regenerate-text")}</p>
            <form on:submit=regenerate>
                <label for="codes-password">{move || t("field-password")}</label>
                <input id="codes-password" type="password" autocomplete="current-password" required bind:value=codes_password />
                <label for="codes-code">{move || t("field-code")}</label>
                <input id="codes-code" inputmode="numeric" autocomplete="one-time-code" required bind:value=codes_code />
                <ErrorMessage error=codes_error />
                <button type="submit" disabled=move || busy.get()>{move || t("codes-regenerate")}</button>
            </form>
            {move || codes.get().map(|c| view! { <RecoveryCodes codes=c /> })}
        </section>
    }
}
