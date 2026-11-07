//! Shared pieces of the pages: session state, roles, error messages and small components.

use leptos::prelude::*;
use serde_json::Value;

use crate::api::ApiError;
use crate::i18n::{t, t_with};

/// The signed-in administrator (`GET /api/v1/me`), or `None`.
#[derive(Clone, Copy)]
pub struct Session {
    pub me: RwSignal<Option<Value>>,
    /// A message shown once at the top of the next page (a translation key).
    pub notice: RwSignal<Option<String>>,
}

pub fn session() -> Session {
    expect_context::<Session>()
}

/// A field as text: strings bare, `null` as an empty string.
pub fn text(value: &Value, pointer: &str) -> String {
    match value.pointer(pointer) {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

pub fn items(value: &Value) -> Vec<Value> {
    value
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub fn is_instance_owner(me: &Value) -> bool {
    me.get("is_instance_owner").and_then(Value::as_bool) == Some(true)
}

pub fn role_in(me: &Value, organization: &str) -> Option<String> {
    me.pointer(&format!("/roles/{organization}"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// May rename the organization, invite and change roles (permission matrix).
pub fn manages(me: &Value, organization: &str) -> bool {
    is_instance_owner(me) || role_in(me, organization).as_deref() == Some("owner")
}

/// May read the audit somewhere.
pub fn reads_audit(me: &Value) -> bool {
    is_instance_owner(me)
        || me
            .get("roles")
            .and_then(Value::as_object)
            .is_some_and(|roles| {
                roles
                    .values()
                    .any(|r| matches!(r.as_str(), Some("owner" | "auditor")))
            })
}

pub const ROLES: [&str; 5] = ["owner", "administrator", "approver", "auditor", "read_only"];

pub fn role_name(role: &str) -> String {
    t(&format!("role-{}", role.replace('_', "-")))
}

/// A translated message for an API error; the Hub's own message is added for input errors,
/// where it names the field.
pub fn error_text(error: &ApiError) -> String {
    let key = format!("error-{}", error.code.replace('_', "-"));
    let mut message = t(&key);
    if message == key {
        message = t("error-unexpected");
    }
    if matches!(error.code.as_str(), "invalid" | "conflict") && !error.message.is_empty() {
        message = t_with(
            "error-with-detail",
            &[("message", message), ("detail", error.message.clone())],
        );
    }
    message
}

/// Local time of an RFC 3339 instant, in the console's language.
pub fn when(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.to_owned();
    }
    let language = crate::i18n::language().get_untracked();
    String::from(date.to_locale_string(&language, &wasm_bindgen::JsValue::UNDEFINED))
}

/// The error of the last action, announced to screen readers.
#[component]
pub fn ErrorMessage(error: RwSignal<Option<ApiError>>) -> impl IntoView {
    view! {
        <Show when=move || error.get().is_some()>
            <p class="error" role="alert">{move || error.get().map(|e| error_text(&e))}</p>
        </Show>
    }
}

/// Page title, focused when the page opens so screen readers announce it.
#[component]
pub fn Title(key: &'static str) -> impl IntoView {
    view! { <h1 id="page-title" tabindex="-1">{move || t(key)}</h1> }
}

/// Runs an API call and stores its error; `then` gets the result on success.
pub fn run<F, T>(
    busy: RwSignal<bool>,
    error: RwSignal<Option<ApiError>>,
    call: F,
    then: impl FnOnce(T) + 'static,
) where
    F: std::future::Future<Output = Result<T, ApiError>> + 'static,
    T: 'static,
{
    busy.set(true);
    error.set(None);
    leptos::task::spawn_local(async move {
        let result = call.await;
        busy.set(false);
        match result {
            Ok(value) => then(value),
            Err(e) => error.set(Some(e)),
        }
    });
}

/// The data URL of a QR code for an `otpauth://` URI.
pub fn qr_data_url(uri: &str) -> Option<String> {
    let code = qrcode::QrCode::new(uri.as_bytes()).ok()?;
    let svg = code
        .render::<qrcode::render::svg::Color<'_>>()
        .min_dimensions(200, 200)
        .quiet_zone(true)
        .build();
    Some(format!(
        "data:image/svg+xml;base64,{}",
        data_encoding::BASE64.encode(svg.as_bytes())
    ))
}
