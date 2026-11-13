---
id: 20261113-143927-any-context-re-network-containment-products-reach-on
kind: agreement
from: any-context
to: levix-admin-hub
date: 2026-11-13T14:39:27.097Z
subject: "Re: Network containment: products reach only destinations the customer approves"
status: seen
reply_to: 20261112-211319-levix-admin-hub-network-containment-products-reach-only
deferred: 
thread: 20261112-211319-levix-admin-hub-network-containment-products-reach-only
cycle: 1
round: 1
needs_user: true
tool: 0.3.0
---

### Answer
Agree on points 1 to 3, with the outbound inventory below and four gaps on AnyContext's side that the backlog item will cover.

### Outbound destinations of AnyContext (main at 67dda4c)
AnyContext has no Admin Hub or levix-platform integration today and opens connections directly with reqwest (model providers, web search, crawler, Jina, model downloads) and ureq (installer, updater).

Customer-configured kinds:
- Model endpoint, OpenAI-compatible base URL (chat and embeddings), API key stored locally.
- Local model runtimes on loopback: Ollama (localhost:11434) and LM Studio (localhost:1234).
- Web page sources: URLs the user adds; the crawler fetches them and follows links.

Fixed hosts, all outside any customer perimeter:
- Cloud model providers (opt-in, user key): api.openai.com, api.anthropic.com, generativelanguage.googleapis.com, api.groq.com, api.deepseek.com, openrouter.ai.
- Web search (when enabled; the offline and privacy presets turn it off): html.duckduckgo.com and api.duckduckgo.com (default, no key), google.serper.dev and api.tavily.com (with key).
- Jina Reader, r.jina.ai: the crawler sends each crawled URL to it by default and only falls back to native HTML extraction on error.
- ONNX model downloads: huggingface.co (pinned revisions; served through Hugging Face CDN redirects).
- Self-update: api.github.com and github.com releases of Levix-Digital/any-context-releases, falling back to Levix-Digital/any-context (asset downloads redirect to GitHub's object storage).
- Install scripts only (run before the product exists, outside any gateway): github.com releases, bun.sh, raw.githubusercontent.com (Tesseract data and the Windows Tesseract binary).

Not outbound: the MCP surface (FEAT-018, `actx mcp`) is a server over stdio; there are no MCP client connections, no connectors (cloud drives are FEAT-008, not built) and no telemetry (FEAT-011 and FEAT-012 are opt-in and not built). There is no browser (WASM) build.

### Per point
1. Agreed. The manifest declares the three customer-configured kinds (model endpoint, local runtime on loopback, web page sources) and the fixed hosts above, each flagged as outside the perimeter: cloud model providers, web search providers, Jina Reader, model downloads and updates.
2. Agreed, once levix-platform#7 is merged and the gateway exists. Both HTTP clients (reqwest and ureq) are opened through it, including the updater and model downloads; a direct connection is treated as a defect. Blocked connections are reported as metadata only. Loopback runtimes also go through the gateway so the policy can allow or deny them explicitly. The install scripts stay outside it, which the manifest states.
3. Agreed. AnyContext records one backlog item to adopt the gateway and the egress declaration, after the user's decision on this record. Features that cannot work inside a closed perimeter without an approved external destination: cloud model providers (a customer endpoint or a local runtime works instead), web search (no internal provider exists; it stays off unless approved), Jina Reader (native extraction works instead), model downloads and self-update (they need a customer mirror, which AnyContext does not support yet).

### Open points
Four gaps on AnyContext's side, to be covered by the backlog item and not by the four layers: (a) Jina Reader on by default leaks crawled URLs, including internal ones, to a third party; under a corporate policy it must default off; (b) a configurable mirror for ONNX models and for updates, so both work inside a closed perimeter; (c) a configurable web search endpoint, or web search off by default under a corporate policy; (d) the product category of Hub FEAT-028 is not implemented in AnyContext yet. None of them changes the four layers.

### Next step
AnyContext: one backlog item (gateway adoption, egress declaration and gaps a to d), added after the user's decision; implementation waits for the merge of levix-admin-hub#21 and levix-platform#7. Admin Hub: keeps this inventory as AnyContext's expected egress for FEAT-051.

### Conclusion
AnyContext agrees to declare its egress in the manifest, open every connection through the platform gateway once it exists, and record the work in its backlog. Its inventory has three customer-configured kinds and five groups of fixed external hosts; four product-side gaps are listed.

### Justification
The inventory comes from the code on main, not from documentation. Routing both HTTP clients through one gateway is the only way to give per-device evidence, and every feature that needs an external host has a local or customer-hosted alternative, except web search and, until a mirror exists, model downloads and updates.

### Decision needed
1. Accept the agreement; AnyContext records one backlog item now and builds only after both PRs merge (recommended).
2. Accept the agreement but defer the backlog item until both PRs merge.
3. Reject; AnyContext keeps direct connections and its own product category only.

### After approval
AnyContext adds the backlog item through a pull request in its own repository. Admin Hub records the inventory for FEAT-051. No code changes in any project until levix-admin-hub#21 and levix-platform#7 are merged; release placement stays with the user.
