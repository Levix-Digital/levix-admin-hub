//! The organizations the administrator can see, and creating one (instance owner).

use leptos::prelude::*;
use serde_json::{json, Value};

use crate::api::{self, ApiError};
use crate::i18n::t;
use crate::route::Route;
use crate::ui::{
    is_instance_owner, items, role_in, role_name, run, session, text, ErrorMessage, Title,
};

#[component]
pub fn Organizations() -> impl IntoView {
    let session = session();
    let list = RwSignal::new(None::<Vec<Value>>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);
    let slug = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let create_error = RwSignal::new(None::<ApiError>);

    let load = move || {
        run(busy, error, api::get("/api/v1/organizations"), move |v| {
            list.set(Some(items(&v)))
        });
    };
    load();

    let create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = json!({"slug": slug.get_untracked(), "name": name.get_untracked()});
        run(
            busy,
            create_error,
            async move { api::post("/api/v1/organizations", Some(&body)).await },
            move |_| {
                slug.set(String::new());
                name.set(String::new());
                load();
            },
        );
    };

    let owner = move || session.me.get().as_ref().is_some_and(is_instance_owner);

    view! {
        <section>
            <Title key="orgs-title" />
            <ErrorMessage error=error />
            {move || list.get().map(|orgs| {
                if orgs.is_empty() {
                    return view! { <p>{move || t("orgs-empty")}</p> }.into_any();
                }
                let me = session.me.get().unwrap_or_default();
                view! {
                    <table>
                        <caption class="visually-hidden">{move || t("orgs-title")}</caption>
                        <thead><tr>
                            <th scope="col">{move || t("col-name")}</th>
                            <th scope="col">{move || t("col-slug")}</th>
                            <th scope="col">{move || t("col-status")}</th>
                            <th scope="col">{move || t("col-your-role")}</th>
                        </tr></thead>
                        <tbody>
                            {orgs.into_iter().map(|org| {
                                let id = text(&org, "/id");
                                let status = text(&org, "/status");
                                let role = role_in(&me, &id);
                                view! { <tr>
                                    <td><a href=Route::Organization(id.clone()).href()>{text(&org, "/name")}</a></td>
                                    <td><code>{text(&org, "/slug")}</code></td>
                                    <td>{move || t(&format!("org-status-{status}"))}</td>
                                    <td>{move || role.as_deref().map(role_name).unwrap_or_else(|| t("role-instance-owner"))}</td>
                                </tr> }
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_any()
            })}
            <Show when=owner>
                <h2>{move || t("orgs-create-title")}</h2>
                <form on:submit=create>
                    <label for="org-name">{move || t("field-org-name")}</label>
                    <input id="org-name" required bind:value=name />
                    <label for="org-slug">{move || t("field-slug")}</label>
                    <input id="org-slug" required pattern="[a-z0-9][a-z0-9-]*" aria-describedby="org-slug-help" bind:value=slug />
                    <p id="org-slug-help" class="help">{move || t("field-slug-help")}</p>
                    <ErrorMessage error=create_error />
                    <button type="submit" disabled=move || busy.get()>{move || t("orgs-create")}</button>
                </form>
            </Show>
        </section>
    }
}
