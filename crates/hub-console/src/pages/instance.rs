//! The instance and its session limits (instance owner).

use leptos::prelude::*;
use serde_json::{json, Value};

use crate::api::{self, ApiError};
use crate::i18n::t;
use crate::ui::{run, text, when, ErrorMessage, Title};

#[component]
pub fn Instance() -> impl IntoView {
    let instance = RwSignal::new(None::<Value>);
    let inactivity = RwSignal::new(String::new());
    let absolute = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);
    let saved = RwSignal::new(false);

    let show = move |v: Value| {
        inactivity.set(text(&v, "/settings/session_inactivity_minutes"));
        absolute.set(text(&v, "/settings/session_absolute_hours"));
        instance.set(Some(v));
    };
    run(busy, error, api::get("/api/v1/instance/settings"), show);

    let save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        saved.set(false);
        let body = json!({
            "session_inactivity_minutes": inactivity.get_untracked().trim().parse::<u32>().unwrap_or(0),
            "session_absolute_hours": absolute.get_untracked().trim().parse::<u32>().unwrap_or(0),
        });
        run(
            busy,
            error,
            async move { api::put("/api/v1/instance/settings", &body).await },
            move |v| {
                show(v);
                saved.set(true);
            },
        );
    };

    view! {
        <section>
            <Title key="instance-title" />
            {move || instance.get().map(|i| view! {
                <dl>
                    <dt>{move || t("instance-id")}</dt><dd><code>{text(&i, "/id")}</code></dd>
                    <dt>{move || t("instance-initialized")}</dt><dd>{when(&text(&i, "/initialized_at"))}</dd>
                </dl>
            })}
            <h2>{move || t("instance-sessions")}</h2>
            <form on:submit=save>
                <label for="inst-inactivity">{move || t("field-inactivity")}</label>
                <input id="inst-inactivity" type="number" min="5" max="60" required bind:value=inactivity />
                <label for="inst-absolute">{move || t("field-absolute")}</label>
                <input id="inst-absolute" type="number" min="1" max="24" required bind:value=absolute />
                <ErrorMessage error=error />
                <Show when=move || saved.get()><p role="status">{move || t("saved")}</p></Show>
                <button type="submit" disabled=move || busy.get()>{move || t("save")}</button>
            </form>
        </section>
    }
}
