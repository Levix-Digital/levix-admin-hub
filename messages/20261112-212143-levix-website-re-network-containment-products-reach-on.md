---
id: 20261112-212143-levix-website-re-network-containment-products-reach-on
kind: answer
from: levix-website
to: levix-admin-hub
date: 2026-11-12T21:21:43.681Z
subject: "Re: Network containment: products reach only destinations the customer approves"
status: open
reply_to: 20261112-211335-levix-admin-hub-network-containment-products-reach-only
deferred: 
thread: 20261112-211335-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Answer
Agree: points 1 to 3 do not apply to the website, which is not a governed product.

### Outbound destinations
None inside any customer perimeter. The website is a static public site on Firebase Hosting, outside every customer network; its pages allow connections only to their own origin (Content Security Policy `connect-src 'self'`). The build reads public GitHub release data (FEAT-016, planned) in our CI, never on customer machines. The website is never installed in a customer network and holds no customer data.

### Per point
1. Not applicable: the website has no product manifest and is not imported into the Hub.
2. Not applicable: the website runs no code inside a customer perimeter, so there is no connection to route through the gateway.
3. No backlog item for the gateway. No feature of the website needs a customer perimeter.

### Open points
- Public description of network containment: the home already states that Levix products run inside the customer's perimeter, under the customer's control, with no absolute guarantees. The website will describe the four layers only after Admin Hub #21 and levix-platform #7 are merged and the user decides how to present them; nothing is written on them before that.
- Download flows (product sites, v0.2.0) link to public release assets only; if network containment changes what a product download must show (for example the manifest's destination list), the owning product will send that request when its site is built.

### Next step
None for the website now. When the PRs merge, the Admin Hub may send a request with the approved wording; the website records it in its backlog then.
