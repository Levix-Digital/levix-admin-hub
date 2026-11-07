//! One organization: its details, its members and their roles, invitations and account
//! actions on members.

use leptos::prelude::*;
use serde_json::{json, Value};

use crate::api::{self, segment, ApiError};
use crate::i18n::{t, t_with};
use crate::ui::{items, manages, role_name, run, session, text, when, ErrorMessage, Title, ROLES};

fn confirm(message: &str) -> bool {
    web_sys::window()
        .and_then(|w| w.confirm_with_message(message).ok())
        .unwrap_or(false)
}

fn link_address(token: &str) -> String {
    let origin = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    format!("{origin}/#/link/{token}")
}

#[component]
pub fn Organization(id: String) -> impl IntoView {
    let session = session();
    let id = segment(&id);
    let org = RwSignal::new(None::<Value>);
    let members = RwSignal::new(Vec::<Value>::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<ApiError>);
    let action_error = RwSignal::new(None::<ApiError>);
    let issued_link = RwSignal::new(None::<(String, String)>);

    let base = format!("/api/v1/organizations/{id}");
    let load = {
        let base = base.clone();
        move || {
            let organization = base.clone();
            run(
                busy,
                error,
                async move { api::get(&organization).await },
                move |v| org.set(Some(v)),
            );
            let path = format!("{base}/members");
            run(
                busy,
                error,
                async move { api::get(&path).await },
                move |v| members.set(items(&v)),
            );
        }
    };
    load();

    let can_manage = {
        let id = id.clone();
        move || session.me.get().is_some_and(|me| manages(&me, &id))
    };

    // Rename and archive.
    let new_name = RwSignal::new(String::new());
    let rename = {
        let base = base.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let path = base.clone();
            let body = json!({"name": new_name.get_untracked()});
            run(
                busy,
                action_error,
                async move { api::patch(&path, &body).await },
                move |v| {
                    new_name.set(String::new());
                    org.set(Some(v));
                },
            );
        }
    };
    let archive = {
        let base = base.clone();
        move |_| {
            if !confirm(&t("org-archive-confirm")) {
                return;
            }
            let path = format!("{base}/archive");
            run(
                busy,
                action_error,
                async move { api::post(&path, None).await },
                move |v| org.set(Some(v)),
            );
        }
    };

    // Invite or add a member.
    let email = RwSignal::new(String::new());
    let role = RwSignal::new("read_only".to_owned());
    let add = {
        let base = base.clone();
        let load = load.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            issued_link.set(None);
            let path = format!("{base}/members");
            let body = json!({"email": email.get_untracked(), "role": role.get_untracked()});
            let load = load.clone();
            run(
                busy,
                action_error,
                async move { api::post(&path, Some(&body)).await },
                move |v: Value| {
                    if let Some(token) = v.pointer("/invitation/token").and_then(Value::as_str) {
                        issued_link.set(Some((
                            t_with("org-invitation-issued", &[("email", email.get_untracked())]),
                            link_address(token),
                        )));
                    }
                    email.set(String::new());
                    load();
                },
            );
        }
    };

    let member_row = {
        let base = base.clone();
        let load = load.clone();
        let can_manage = can_manage.clone();
        move |member: Value| {
            let member_id = text(&member, "/id");
            let current_role = text(&member, "/role");
            let status = text(&member, "/status");
            let is_self = session
                .me
                .get_untracked()
                .is_some_and(|me| text(&me, "/id") == member_id);
            let chosen = RwSignal::new(current_role.clone());
            let member_path = format!("{base}/members/{}", segment(&member_id));
            let account_path = format!("/api/v1/administrators/{}", segment(&member_id));
            let label = text(&member, "/email");

            let change_role = {
                let path = member_path.clone();
                let load = load.clone();
                move |_| {
                    let path = path.clone();
                    let body = json!({"role": chosen.get_untracked()});
                    let load = load.clone();
                    run(
                        busy,
                        action_error,
                        async move { api::put(&path, &body).await },
                        move |_| load(),
                    );
                }
            };
            let remove = {
                let path = member_path.clone();
                let load = load.clone();
                let label = label.clone();
                move |_| {
                    if !confirm(&t_with(
                        "member-remove-confirm",
                        &[("email", label.clone())],
                    )) {
                        return;
                    }
                    let path = path.clone();
                    let load = load.clone();
                    run(
                        busy,
                        action_error,
                        async move { api::delete(&path).await },
                        move |_| load(),
                    );
                }
            };
            let account = {
                let load = load.clone();
                let label = label.clone();
                move |action: &'static str| {
                    let path = format!("{account_path}/{action}");
                    let load = load.clone();
                    let label = label.clone();
                    move |_| {
                        if !confirm(&t_with(
                            &format!("account-{action}-confirm"),
                            &[("email", label.clone())],
                        )) {
                            return;
                        }
                        let path = path.clone();
                        let load = load.clone();
                        let label = label.clone();
                        run(
                            busy,
                            action_error,
                            async move { api::post(&path, None).await },
                            move |v: Value| {
                                if action == "reset-second-factor" {
                                    if let Some(token) = v.get("token").and_then(Value::as_str) {
                                        issued_link.set(Some((
                                            t_with("account-reset-issued", &[("email", label)]),
                                            link_address(token),
                                        )));
                                    }
                                }
                                load();
                            },
                        );
                    }
                }
            };
            let manage = can_manage() && !is_self;
            let select_id = format!("role-{member_id}");
            let suspended = status == "suspended";
            view! {
                <tr>
                    <td>{
                        let name = text(&member, "/display_name");
                        (!name.is_empty() && name != label).then(|| view! { {name}<br /> })
                    }<span class="muted">{label.clone()}</span></td>
                    <td>{
                        let current_role = current_role.clone();
                        move || role_name(&current_role)
                    }</td>
                    <td>{move || t(&format!("admin-status-{status}"))}</td>
                    <td>
                        <Show when=move || manage><div class="actions">
                            <label class="visually-hidden" for=select_id.clone()>
                                {let label = label.clone(); move || t_with("member-role-for", &[("email", label.clone())])}
                            </label>
                            <select id=select_id.clone() on:change=move |ev| chosen.set(event_target_value(&ev))
                                prop:value=move || chosen.get()>
                                {ROLES.iter().map(|r| view! { <option value=*r>{move || role_name(r)}</option> }).collect_view()}
                            </select>
                            <button type="button" class="secondary" on:click=change_role.clone()>{move || t("member-change-role")}</button>
                            <button type="button" class="secondary" on:click=remove.clone()>{move || t("member-remove")}</button>
                            {if suspended {
                                view! { <button type="button" class="secondary" on:click=account("reactivate")>{move || t("account-reactivate")}</button> }.into_any()
                            } else {
                                view! { <button type="button" class="secondary" on:click=account("suspend")>{move || t("account-suspend")}</button> }.into_any()
                            }}
                            <button type="button" class="secondary" on:click=account("reset-second-factor")>{move || t("account-reset-second-factor")}</button>
                            <button type="button" class="secondary" on:click=account("revoke-sessions")>{move || t("account-revoke-sessions")}</button>
                        </div></Show>
                    </td>
                </tr>
            }
        }
    };

    view! {
        <section>
            <Title key="org-title" />
            <ErrorMessage error=error />
            {move || org.get().map(|o| {
                let status = text(&o, "/status");
                view! {
                    <h2 class="org-name">{text(&o, "/name")}</h2>
                    <dl>
                        <dt>{move || t("col-slug")}</dt><dd><code>{text(&o, "/slug")}</code></dd>
                        <dt>{move || t("col-status")}</dt><dd>{move || t(&format!("org-status-{status}"))}</dd>
                        <dt>{move || t("col-created")}</dt><dd>{when(&text(&o, "/created_at"))}</dd>
                    </dl>
                }
            })}
            <ErrorMessage error=action_error />
            {move || issued_link.get().map(|(message, address)| view! {
                <div class="notice" role="status">
                    <p>{message}</p>
                    <p><code class="secret" data-testid="issued-link">{address}</code></p>
                    <p class="help">{move || t("link-deliver-help")}</p>
                </div>
            })}

            <h2>{move || t("members-title")}</h2>
            <table>
                <caption class="visually-hidden">{move || t("members-title")}</caption>
                <thead><tr>
                    <th scope="col">{move || t("col-administrator")}</th>
                    <th scope="col">{move || t("col-role")}</th>
                    <th scope="col">{move || t("col-status")}</th>
                    <th scope="col">{move || t("col-actions")}</th>
                </tr></thead>
                <tbody>
                    {move || members.get().into_iter().map(member_row.clone()).collect_view()}
                </tbody>
            </table>

            <Show when=can_manage.clone()>
                <h2>{move || t("members-add-title")}</h2>
                <form on:submit=add.clone()>
                    <label for="member-email">{move || t("field-email")}</label>
                    <input id="member-email" type="email" required bind:value=email />
                    <label for="member-role">{move || t("col-role")}</label>
                    <select id="member-role" on:change=move |ev| role.set(event_target_value(&ev)) prop:value=move || role.get()>
                        {ROLES.iter().map(|r| view! { <option value=*r>{move || role_name(r)}</option> }).collect_view()}
                    </select>
                    <button type="submit" disabled=move || busy.get()>{move || t("members-add")}</button>
                </form>

                <h2>{move || t("org-manage-title")}</h2>
                <form on:submit=rename.clone()>
                    <label for="org-rename">{move || t("field-new-org-name")}</label>
                    <input id="org-rename" required bind:value=new_name />
                    <button type="submit" disabled=move || busy.get()>{move || t("org-rename")}</button>
                </form>
                <p><button type="button" class="danger" on:click=archive.clone()>{move || t("org-archive")}</button></p>
            </Show>
        </section>
    }
}
