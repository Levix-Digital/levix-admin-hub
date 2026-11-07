//! The console shell: start-up checks, header, navigation, language menu and the routed page.

use leptos::prelude::*;
use serde_json::{json, Value};

use crate::api;
use crate::i18n::{self, t, LANGUAGES};
use crate::idle;
use crate::pages;
use crate::route::{self, Route};
use crate::ui::{is_instance_owner, reads_audit, session, Session};

const LANGUAGE_KEY: &str = "levix-hub.language";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Boot {
    Loading,
    NotInitialized,
    Ready,
    Unreachable,
}

/// Applies a language to the page and to the `lang` attribute.
fn apply_language(language: RwSignal<String>, value: &str) {
    language.set(value.to_owned());
    if let Some(element) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ignored = element.set_attribute("lang", value);
    }
}

#[component]
pub fn App() -> impl IntoView {
    let initial = api::preference(LANGUAGE_KEY)
        .and_then(|l| i18n::supported(&l))
        .unwrap_or_else(i18n::from_browser);
    let language = i18n::language();
    apply_language(language, initial);

    let session = Session {
        me: RwSignal::new(None),
        notice: RwSignal::new(None),
    };
    provide_context(session);
    let route = RwSignal::new(route::current());
    let boot = RwSignal::new(Boot::Loading);

    let _hash = window_event_listener(leptos::ev::hashchange, move |_| {
        if let Some(next) = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .and_then(|h| Route::parse(&h))
        {
            route.set(next);
            focus_title();
        }
    });

    api::on_signed_out(move || {
        if session.me.get_untracked().is_some() {
            session.me.set(None);
            api::set_csrf(None);
            session.notice.set(Some("notice-session-ended".to_owned()));
        }
    });

    // The profile language wins once signed in.
    Effect::new(move |_| {
        if let Some(me) = session.me.get() {
            if let Some(locale) = me
                .get("locale")
                .and_then(Value::as_str)
                .and_then(i18n::supported)
            {
                if language.get_untracked() != locale {
                    apply_language(language, locale);
                }
            }
        }
    });

    leptos::task::spawn_local(async move {
        match api::public_get("/api/v1/instance").await {
            Ok(status) if status.get("initialized").and_then(Value::as_bool) == Some(false) => {
                boot.set(Boot::NotInitialized);
                return;
            }
            Ok(_) => {}
            Err(_) => {
                boot.set(Boot::Unreachable);
                return;
            }
        }
        if let Ok(me) = api::get("/api/v1/me").await {
            session.me.set(Some(me));
        }
        boot.set(Boot::Ready);
    });

    idle::watch(session);

    view! {
        <a class="skip-link" href="#main" on:click=move |ev| {
            ev.prevent_default();
            focus_main();
        }>{move || t("skip-to-content")}</a>
        <header class="top">
            <span class="brand">{move || t("app-name")}</span>
            <Navigation route=route />
            <LanguageMenu language=language />
        </header>
        <main id="main" tabindex="-1">
            <idle::Warning session=session />
            {move || session.notice.get().map(|key| view! {
                <div class="notice" role="status">
                    <p>{move || t(&key)}</p>
                    <button type="button" class="secondary" on:click=move |_| session.notice.set(None)>
                        {move || t("dismiss")}
                    </button>
                </div>
            })}
            {move || match boot.get() {
                Boot::Loading => view! { <p role="status">{move || t("loading")}</p> }.into_any(),
                Boot::NotInitialized => view! { <pages::NotInitialized /> }.into_any(),
                Boot::Unreachable => view! { <p class="error" role="alert">{move || t("error-network")}</p> }.into_any(),
                Boot::Ready => view! { <Page route=route /> }.into_any(),
            }}
        </main>
    }
}

fn focus(id: &str) {
    if let Some(element) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
    {
        use wasm_bindgen::JsCast;
        if let Ok(element) = element.dyn_into::<web_sys::HtmlElement>() {
            let _ignored = element.focus();
        }
    }
}

fn focus_main() {
    focus("main");
}

fn focus_title() {
    // After the new page renders.
    gloo_timers::callback::Timeout::new(0, || focus("page-title")).forget();
}

#[component]
fn Page(route: RwSignal<Route>) -> impl IntoView {
    let session = session();
    move || {
        let current = route.get();
        let signed_in = session.me.get().is_some();
        match current {
            Route::Link(token) => view! { <pages::link::LinkPage token=token /> }.into_any(),
            Route::NotFound => view! { <pages::NotFound /> }.into_any(),
            Route::Login if signed_in => {
                route::go(&Route::Organizations);
                view! { <pages::organizations::Organizations /> }.into_any()
            }
            _ if !signed_in => view! { <pages::login::Login /> }.into_any(),
            Route::Home | Route::Login | Route::Organizations => {
                view! { <pages::organizations::Organizations /> }.into_any()
            }
            Route::Organization(id) => {
                view! { <pages::organization::Organization id=id /> }.into_any()
            }
            Route::Profile => view! { <pages::profile::Profile /> }.into_any(),
            Route::Sessions => view! { <pages::sessions::Sessions /> }.into_any(),
            Route::Tokens => view! { <pages::tokens::Tokens /> }.into_any(),
            Route::Audit => view! { <pages::audit::Audit /> }.into_any(),
            Route::Instance => view! { <pages::instance::Instance /> }.into_any(),
        }
    }
}

#[component]
fn Navigation(route: RwSignal<Route>) -> impl IntoView {
    let session = session();
    let link = move |target: Route, key: &'static str| {
        let href = target.href();
        let active = {
            let target = target.clone();
            move || {
                let current = route.get();
                current == target
                    || matches!(
                        (&current, &target),
                        (Route::Organization(_), Route::Organizations)
                    )
            }
        };
        view! {
            <li><a href=href aria-current=move || active().then_some("page")>{move || t(key)}</a></li>
        }
    };
    let sign_out = move |_| {
        leptos::task::spawn_local(async move {
            let _ignored = api::post("/api/v1/auth/logout", None).await;
            api::set_csrf(None);
            session.me.set(None);
            session.notice.set(Some("notice-signed-out".to_owned()));
            route::go(&Route::Login);
        });
    };
    move || {
        session.me.get().map(|me| {
            let owner = is_instance_owner(&me);
            let audit = reads_audit(&me);
            view! {
                <nav aria-label=move || t("nav-label")>
                    <ul>
                        {link(Route::Organizations, "nav-organizations")}
                        {audit.then(|| link(Route::Audit, "nav-audit"))}
                        {link(Route::Profile, "nav-profile")}
                        {link(Route::Sessions, "nav-sessions")}
                        {link(Route::Tokens, "nav-tokens")}
                        {owner.then(|| link(Route::Instance, "nav-instance"))}
                    </ul>
                </nav>
                <span class="who">{me.get("email").and_then(Value::as_str).unwrap_or_default().to_owned()}</span>
                <button type="button" class="secondary" on:click=sign_out>{move || t("sign-out")}</button>
            }
        })
    }
}

#[component]
fn LanguageMenu(language: RwSignal<String>) -> impl IntoView {
    let session = session();
    let change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        let Some(chosen) = i18n::supported(&value) else {
            return;
        };
        apply_language(language, chosen);
        api::set_preference(LANGUAGE_KEY, chosen);
        if session.me.get_untracked().is_some() {
            leptos::task::spawn_local(async move {
                if let Ok(me) = api::patch("/api/v1/me", &json!({"locale": chosen})).await {
                    session.me.set(Some(me));
                }
            });
        }
    };
    view! {
        <label class="language">
            <span>{move || t("language")}</span>
            <select on:change=change prop:value=move || language.get()>
                {LANGUAGES.iter().map(|(code, name)| view! {
                    <option value=*code lang=*code>{*name}</option>
                }).collect_view()}
            </select>
        </label>
    }
}
