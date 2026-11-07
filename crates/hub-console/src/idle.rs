//! Inactivity sign-out: the console follows the instance's inactivity limit, warns a minute
//! before it, and signs out when it passes. Real activity keeps the Hub session alive.

use leptos::prelude::*;
use serde_json::Value;

use crate::api;
use crate::i18n::t;
use crate::route::{self, Route};
use crate::ui::Session;

const TICK_MS: u32 = 5_000;
const WARNING_MS: f64 = 60_000.0;
/// Activity refreshes the Hub session at most this often.
const KEEPALIVE_MS: f64 = 120_000.0;

#[derive(Clone, Copy)]
struct Idle {
    last_activity: RwSignal<f64>,
    last_contact: RwSignal<f64>,
    limit_ms: RwSignal<Option<f64>>,
    warning: RwSignal<bool>,
}

fn now() -> f64 {
    js_sys::Date::now()
}

pub fn watch(session: Session) {
    let idle = Idle {
        last_activity: RwSignal::new(now()),
        last_contact: RwSignal::new(now()),
        limit_ms: RwSignal::new(None),
        warning: RwSignal::new(false),
    };
    provide_context(idle);

    let active = move || {
        if !idle.warning.get_untracked() {
            idle.last_activity.set(now());
        }
    };
    let _keys = window_event_listener(leptos::ev::keydown, move |_| active());
    let _pointer = window_event_listener(leptos::ev::pointerdown, move |_| active());
    let _scroll = window_event_listener(leptos::ev::wheel, move |_| active());

    // Read the limit once signed in.
    Effect::new(move |_| {
        if session.me.get().is_some() && idle.limit_ms.get_untracked().is_none() {
            leptos::task::spawn_local(async move {
                if let Ok(instance) = api::get("/api/v1/instance/settings").await {
                    let minutes = instance
                        .pointer("/settings/session_inactivity_minutes")
                        .and_then(Value::as_f64);
                    idle.limit_ms.set(minutes.map(|m| m * 60_000.0));
                }
            });
            idle.last_activity.set(now());
            idle.last_contact.set(now());
        }
    });

    gloo_timers::callback::Interval::new(TICK_MS, move || {
        if session.me.get_untracked().is_none() {
            idle.warning.set(false);
            return;
        }
        let Some(limit) = idle.limit_ms.get_untracked() else {
            return;
        };
        let quiet = now() - idle.last_activity.get_untracked();
        if quiet >= limit {
            idle.warning.set(false);
            leptos::task::spawn_local(async move {
                let _ignored = api::post("/api/v1/auth/logout", None).await;
                api::set_csrf(None);
                session.me.set(None);
                session.notice.set(Some("notice-inactivity".to_owned()));
                route::go(&Route::Login);
            });
        } else if quiet >= limit - WARNING_MS {
            idle.warning.set(true);
        } else if idle.last_activity.get_untracked() > idle.last_contact.get_untracked()
            && now() - idle.last_contact.get_untracked() >= KEEPALIVE_MS
        {
            idle.last_contact.set(now());
            leptos::task::spawn_local(async move {
                if let Ok(me) = api::get("/api/v1/me").await {
                    session.me.set(Some(me));
                }
            });
        }
    })
    .forget();
}

/// The warning shown a minute before the inactivity sign-out.
#[component]
pub fn Warning(session: Session) -> impl IntoView {
    let idle = expect_context::<Idle>();
    let stay = move |_| {
        idle.warning.set(false);
        idle.last_activity.set(now());
        idle.last_contact.set(now());
        leptos::task::spawn_local(async move {
            if let Ok(me) = api::get("/api/v1/me").await {
                session.me.set(Some(me));
            }
        });
    };
    view! {
        <Show when=move || idle.warning.get()>
            <div class="warning" role="alertdialog" aria-labelledby="idle-title" aria-describedby="idle-text">
                <h2 id="idle-title">{move || t("idle-title")}</h2>
                <p id="idle-text">{move || t("idle-text")}</p>
                <button type="button" on:click=stay>{move || t("idle-stay")}</button>
            </div>
        </Show>
    }
}
