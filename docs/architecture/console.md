# Web console

The browser interface of the Hub (crate `hub-console`), embedded in the `levix-hub` binary. It
calls only the public API, so it can do nothing the API does not allow
([ADR-0002](../adr/0002-rust-service-on-tokio-and-axum-with-a-leptos-web-console.md)).

## Build and serving

- A Leptos application compiled to WebAssembly with `trunk` into `crates/hub-console/dist`.
  `hub-api` embeds that directory at compile time; a build without it serves the API only.
- The console is the router's fallback for `GET` and `HEAD`. `index.html` is served `no-cache`;
  the hashed assets are immutable.
- A strict Content Security Policy allows only the console's own files, WebAssembly, and the
  SHA-256 hash of trunk's inline loader script, computed when the server starts (T-04).

## Routing and pages

Pages live in the URL fragment, so tokens in links (`#/link/<token>`) never reach a server or
proxy log. The pages are:

- **Sign-in**, and **Link** for setup, invitations, resets and host recovery (enrollment with a
  QR code and recovery codes).
- **Organizations** and **Organization**: create, rename and archive organizations; add members
  with a role (which shows their invitation link), change roles, remove members; suspend or
  reactivate an account and reset its second factor (which shows the reset link).
- **Profile** (name, language, password), **Sessions** and **Tokens** of the signed-in
  administrator.
- **Audit**, with filters and the verification of the chain.
- **Instance** settings, for instance owners.

## Session handling

- The console signs in with the session cookie and keeps the CSRF token in the browser's local
  storage, sending it on every write.
- An idle watcher warns one minute before the inactivity limit, keeps the session alive while
  the administrator is active, and signs out at the limit.
- Operations that ask for the password again (changing the password, new recovery codes) treat
  a refusal as a wrong password, not as the end of the session, unless the session has really
  ended.

## Languages and accessibility

- Texts are Fluent files in `locales/`: English, French and Spanish. The language comes from the
  administrator's profile, otherwise from the browser, otherwise English.
- Pages target WCAG 2.2 AA. `console-tests/` runs the browser flows with Playwright and an
  axe-core scan of every page in the three languages, in CI.
