# Device channel, version 1

The channel between devices (and product servers) and the Admin Hub. The Hub serves it from v0.2.0 (enrollment, FEAT-006). It is specified in v0.1.0 so products can build their client first (FR-012, ADR-0006). The machine-readable form is [asyncapi.json](asyncapi.json) (AsyncAPI 3.0) with the JSON Schemas in [schemas/](schemas/) and one example per message in [examples/](examples/). The examples show the format only: their nonces and signatures are placeholders.

## Connection

- The device opens `wss://<hub>/ws/device/v1`. The Hub never connects to a device.
- TLS is required. The device checks the Hub certificate against its trust store or the certificate fingerprint given at enrollment.
- Every frame is a UTF-8 JSON text frame holding one envelope. Frames over 64 KiB are refused.

## Envelope

| Member | Meaning |
|---|---|
| `v` | Major version of the channel, always `1` on `/ws/device/v1`. |
| `type` | `hello`, `challenge`, `authenticate`, `welcome` or `error`; selects the payload schema. |
| `id` | UUID v7 of the message. A receiver refuses an identifier it already saw in the session (replay). |
| `ts` | Creation time, RFC 3339 in UTC. A receiver refuses messages more than 5 minutes away from its clock. |
| `payload` | The body, defined by the schema of the type. |
| `sig` | Signature of the Hub. Required on every message from the Hub, absent on messages from the device. |

Every schema refuses unknown members, so no content travels on the channel by accident (ADR-0004, threat model T-34).

## Handshake

1. Device → Hub `hello`: its `device_id`, the minor versions it speaks (`versions`, such as `["1.0"]`), its product and version, and the fingerprint of the Hub key it pinned at enrollment (`hub_key`).
2. Hub → device `challenge` (signed): a fresh 32-byte `nonce`, the chosen `version` and the `hub_key` that signs. The device verifies the signature with the pinned key and closes the connection if the key or the signature does not match (threat model T-31).
3. Device → Hub `authenticate`: an Ed25519 signature by the device key over the UTF-8 bytes of

   ```text
   levix-channel-v1|authenticate|<nonce>|<device_id>|<hub_key>
   ```

   The prefix separates this signature from any other use of the device key (threat model T-30).
4. Hub → device `welcome` (signed): the session identifier, the version in force and the heartbeat interval. Or `error` (signed), then the Hub closes the connection.

The Hub sends `error` with `unsupported_version` when no minor version is common, `unknown_device` or `revoked` for a device it does not accept, `bad_signature` when the proof fails, `rate_limited` with `retry_after_secs` when a device reconnects too often, and `invalid_message` for anything that breaks the schemas.

## Hub signature

- Algorithm: Ed25519 (RFC 8032) with the Hub signing key (FEAT-006, custody FEAT-044, rotation FEAT-045).
- Signed bytes: the RFC 8785 (JCS) canonical JSON of the whole envelope without its `sig` member.
- `sig.key`: `sha256:` followed by the lowercase hex SHA-256 of the raw 32-byte public key. It must equal the key the device pinned.
- `sig.value`: the 64-byte signature in base64url without padding.

Policies, certificates and commands added in later minor versions are envelopes signed the same way (threat model T-32).

## Versioning

- The major version is in the endpoint path and in `v`. A new major gets a new path (`/ws/device/v2`). Each major stays served for at least two Hub minor releases after its successor ships (ADR-0006).
- Minor versions only add message types or optional members. Both sides agree on one minor version in the handshake: the highest one in both lists.
- Contract versions and their changes are listed in [CHANGELOG.md](../../CHANGELOG.md).
