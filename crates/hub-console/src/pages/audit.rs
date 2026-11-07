//! The administrative audit, newest first, and the check of its hash chain.

use leptos::prelude::*;
use serde_json::Value;

use crate::api::{self, segment, ApiError};
use crate::i18n::{t, t_with};
use crate::ui::{items, run, text, when, ErrorMessage, Title};

const PAGE: u32 = 50;

#[component]
pub fn Audit() -> impl IntoView {
    let records = RwSignal::new(Vec::<Value>::new());
    let next = RwSignal::new(None::<i64>);
    let organizations = RwSignal::new(Vec::<Value>::new());
    let filter = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);
    let report = RwSignal::new(None::<Value>);

    let fetch = move |before: Option<i64>| {
        let mut path = format!("/api/v1/audit?limit={PAGE}");
        let organization = filter.get_untracked();
        if !organization.is_empty() {
            path.push_str(&format!("&organization={}", segment(&organization)));
        }
        if let Some(before) = before {
            path.push_str(&format!("&before={before}"));
        }
        run(
            busy,
            error,
            async move { api::get(&path).await },
            move |page: Value| {
                let mut list = if before.is_some() {
                    records.get_untracked()
                } else {
                    Vec::new()
                };
                list.extend(items(&page));
                records.set(list);
                next.set(page.get("next_before").and_then(Value::as_i64));
            },
        );
    };
    fetch(None);
    run(busy, error, api::get("/api/v1/organizations"), move |v| {
        organizations.set(items(&v))
    });

    let verify = move |_| {
        run(busy, error, api::get("/api/v1/audit/verify"), move |v| {
            report.set(Some(v))
        });
    };

    view! {
        <section>
            <Title key="audit-title" />
            <div class="toolbar">
                <label for="audit-org">{move || t("audit-filter")}</label>
                <select id="audit-org" prop:value=move || filter.get() on:change=move |ev| {
                    filter.set(event_target_value(&ev));
                    fetch(None);
                }>
                    <option value="">{move || t("audit-all")}</option>
                    {move || organizations.get().into_iter().map(|o| view! {
                        <option value=text(&o, "/id")>{text(&o, "/name")}</option>
                    }).collect_view()}
                </select>
                <button type="button" class="secondary" on:click=verify>{move || t("audit-verify")}</button>
            </div>
            {move || report.get().map(|r| {
                let intact = r.get("intact").and_then(Value::as_bool) == Some(true);
                let count = text(&r, "/records");
                let broken = text(&r, "/first_broken_seq");
                view! {
                    <p role="status" class=if intact { "ok" } else { "error" }>{move || if intact {
                        t_with("audit-intact", &[("count", count.clone())])
                    } else {
                        t_with("audit-broken", &[("seq", broken.clone())])
                    }}</p>
                }
            })}
            <ErrorMessage error=error />
            <table>
                <caption class="visually-hidden">{move || t("audit-title")}</caption>
                <thead><tr>
                    <th scope="col">"#"</th>
                    <th scope="col">{move || t("col-when")}</th>
                    <th scope="col">{move || t("col-actor")}</th>
                    <th scope="col">{move || t("col-action")}</th>
                    <th scope="col">{move || t("col-target")}</th>
                    <th scope="col">{move || t("col-result")}</th>
                    <th scope="col">{move || t("col-source")}</th>
                </tr></thead>
                <tbody>
                    {move || records.get().into_iter().map(|r| {
                        let result = text(&r, "/result");
                        view! { <tr>
                            <td>{text(&r, "/seq")}</td>
                            <td>{when(&text(&r, "/at"))}</td>
                            <td>{text(&r, "/actor_kind")}" "<span class="muted">{text(&r, "/actor_id")}</span></td>
                            <td><code>{text(&r, "/action")}</code></td>
                            <td>{text(&r, "/target_kind")}" "<span class="muted">{text(&r, "/target_id")}</span></td>
                            <td>{move || t(&format!("result-{result}"))}</td>
                            <td>{text(&r, "/source_address")}</td>
                        </tr> }
                    }).collect_view()}
                </tbody>
            </table>
            {move || next.get().map(|before| view! {
                <button type="button" class="secondary" on:click=move |_| fetch(Some(before))>{move || t("audit-older")}</button>
            })}
        </section>
    }
}
