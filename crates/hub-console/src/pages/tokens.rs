//! Personal API tokens: the secret is shown once, at creation.

use leptos::prelude::*;
use serde_json::{json, Value};

use crate::api::{self, segment, ApiError};
use crate::i18n::t;
use crate::ui::{items, run, text, when, ErrorMessage, Title};

#[component]
pub fn Tokens() -> impl IntoView {
    let list = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);
    let name = RwSignal::new(String::new());
    let days = RwSignal::new("90".to_owned());
    let secret = RwSignal::new(None::<String>);
    let load = move || {
        run(busy, error, api::get("/api/v1/me/tokens"), move |v| {
            list.set(items(&v))
        })
    };
    load();

    let create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = json!({
            "name": name.get_untracked(),
            "days": days.get_untracked().trim().parse::<i64>().unwrap_or(90),
        });
        run(
            busy,
            error,
            async move { api::post("/api/v1/me/tokens", Some(&body)).await },
            move |v: Value| {
                name.set(String::new());
                secret.set(Some(text(&v, "/secret")));
                load();
            },
        );
    };

    view! {
        <section>
            <Title key="tokens-title" />
            <p class="help">{move || t("tokens-text")}</p>
            <ErrorMessage error=error />
            {move || secret.get().map(|s| view! {
                <div class="notice" role="status">
                    <p>{move || t("tokens-secret-once")}</p>
                    <p><code class="secret" data-testid="token-secret">{s}</code></p>
                </div>
            })}
            <form on:submit=create>
                <label for="token-name">{move || t("field-token-name")}</label>
                <input id="token-name" required bind:value=name />
                <label for="token-days">{move || t("field-token-days")}</label>
                <input id="token-days" type="number" min="1" max="90" required bind:value=days />
                <button type="submit" disabled=move || busy.get()>{move || t("tokens-create")}</button>
            </form>
            <table>
                <caption class="visually-hidden">{move || t("tokens-title")}</caption>
                <thead><tr>
                    <th scope="col">{move || t("col-name")}</th>
                    <th scope="col">{move || t("col-prefix")}</th>
                    <th scope="col">{move || t("col-created")}</th>
                    <th scope="col">{move || t("col-expires")}</th>
                    <th scope="col">{move || t("col-actions")}</th>
                </tr></thead>
                <tbody>
                    {move || list.get().into_iter().map(|token| {
                        let id = text(&token, "/id");
                        let revoke = move |_| {
                            let path = format!("/api/v1/me/tokens/{}", segment(&id));
                            run(busy, error, async move { api::delete(&path).await }, move |_| load());
                        };
                        view! { <tr>
                            <td>{text(&token, "/name")}</td>
                            <td><code>{text(&token, "/display_prefix")}</code></td>
                            <td>{when(&text(&token, "/created_at"))}</td>
                            <td>{when(&text(&token, "/expires_at"))}</td>
                            <td><button type="button" class="secondary" on:click=revoke>{move || t("token-revoke")}</button></td>
                        </tr> }
                    }).collect_view()}
                </tbody>
            </table>
        </section>
    }
}
