//! Hash routing: `#/link/<token>` is the address the Hub prints for setup, invitation and
//! recovery links, so every page lives in the fragment, which browsers never send.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Home,
    Login,
    Link(String),
    Organizations,
    Organization(String),
    Profile,
    Sessions,
    Tokens,
    Audit,
    Instance,
    NotFound,
}

impl Route {
    /// The route of a fragment; `None` for fragments that are not routes (`#main`).
    pub fn parse(hash: &str) -> Option<Self> {
        let hash = hash.strip_prefix('#').unwrap_or(hash);
        if hash.is_empty() || hash == "/" {
            return Some(Self::Home);
        }
        let path = hash.strip_prefix('/')?;
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        Some(match parts.as_slice() {
            ["login"] => Self::Login,
            ["link", token] => Self::Link((*token).to_owned()),
            ["organizations"] => Self::Organizations,
            ["organizations", id] => Self::Organization((*id).to_owned()),
            ["profile"] => Self::Profile,
            ["sessions"] => Self::Sessions,
            ["tokens"] => Self::Tokens,
            ["audit"] => Self::Audit,
            ["instance"] => Self::Instance,
            _ => Self::NotFound,
        })
    }

    pub fn href(&self) -> String {
        match self {
            Self::Home => "#/".to_owned(),
            Self::Login => "#/login".to_owned(),
            Self::Link(token) => format!("#/link/{token}"),
            Self::Organizations => "#/organizations".to_owned(),
            Self::Organization(id) => format!("#/organizations/{id}"),
            Self::Profile => "#/profile".to_owned(),
            Self::Sessions => "#/sessions".to_owned(),
            Self::Tokens => "#/tokens".to_owned(),
            Self::Audit => "#/audit".to_owned(),
            Self::Instance => "#/instance".to_owned(),
            Self::NotFound => "#/not-found".to_owned(),
        }
    }
}

pub fn current() -> Route {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .and_then(|h| Route::parse(&h))
        .unwrap_or(Route::Home)
}

pub fn go(route: &Route) {
    if let Some(window) = web_sys::window() {
        let _ignored = window.location().set_hash(&route.href());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragments_map_to_routes() {
        assert_eq!(Route::parse(""), Some(Route::Home));
        assert_eq!(Route::parse("#/"), Some(Route::Home));
        assert_eq!(
            Route::parse("#/link/lvxl_abc"),
            Some(Route::Link("lvxl_abc".to_owned()))
        );
        assert_eq!(
            Route::parse("#/organizations/x"),
            Some(Route::Organization("x".to_owned()))
        );
        assert_eq!(Route::parse("#/nowhere"), Some(Route::NotFound));
        assert_eq!(Route::parse("#main"), None);
        for route in [Route::Login, Route::Audit, Route::Link("t".to_owned())] {
            assert_eq!(Route::parse(&route.href()), Some(route));
        }
    }
}
