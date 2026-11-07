//! The administrator's open sessions.

use leptos::prelude::*;
use serde_json::Value;

use crate::api::{self, segment, ApiError};
use crate::i18n::t;
use crate::ui::{items, run, text, when, ErrorMessage, Title};

#[component]
pub fn Sessions() -> impl IntoView {
    let list = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);
    let load = move || {
        run(busy, error, api::get("/api/v1/me/sessions"), move |v| {
            list.set(items(&v))
        })
    };
    load();

    view! {
        <section>
            <Title key="sessions-title" />
            <ErrorMessage error=error />
            <table>
                <caption class="visually-hidden">{move || t("sessions-title")}</caption>
                <thead><tr>
                    <th scope="col">{move || t("col-kind")}</th>
                    <th scope="col">{move || t("col-created")}</th>
                    <th scope="col">{move || t("col-last-seen")}</th>
                    <th scope="col">{move || t("col-expires")}</th>
                    <th scope="col">{move || t("col-source")}</th>
                    <th scope="col">{move || t("col-actions")}</th>
                </tr></thead>
                <tbody>
                    {move || list.get().into_iter().map(|s| {
                        let id = text(&s, "/id");
                        let kind = text(&s, "/kind");
                        let current = s.get("current").and_then(Value::as_bool) == Some(true);
                        let revoke = move |_| {
                            let path = format!("/api/v1/me/sessions/{}", segment(&id));
                            run(busy, error, async move { api::delete(&path).await }, move |_| load());
                        };
                        view! { <tr>
                            <td>{move || t(&format!("session-kind-{kind}"))}
                                {current.then(|| view! { " " <strong>{move || t("session-current")}</strong> })}</td>
                            <td>{when(&text(&s, "/created_at"))}</td>
                            <td>{when(&text(&s, "/last_seen_at"))}</td>
                            <td>{when(&text(&s, "/expires_at"))}</td>
                            <td>{text(&s, "/source_address")}</td>
                            <td>{(!current).then(|| view! {
                                <button type="button" class="secondary" on:click=revoke.clone()>{move || t("session-revoke")}</button>
                            })}</td>
                        </tr> }
                    }).collect_view()}
                </tbody>
            </table>
        </section>
    }
}
