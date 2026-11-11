# V6.2 experiments

The readings V6.2 takes (`v6.2implementation.md` §15). Each is dated, labelled `d1` where it was read on the founder's machine, and settles the decisions it names with an "As read" note in the plan. Nothing here holds a key, a transcript's text beyond what the repository already holds, or a recording.

## TE-15 - Gemini as the plan's provider

**Setup's note (2026-11-11).** The key answers Google's model list (`GET /v1beta/models`, HTTP 200): it is valid. It is not in the `AIza…` form, so the gate searches for its exact value rather than a pattern (`scripts/v62-gate.sh`, check 3).

### First read (P183, 2026-11-11, d1)

**Models the key sees** (62 in all; text models with `generateContent`, all with thinking): `gemini-2.5-flash`, `gemini-2.5-flash-lite`, `gemini-2.5-pro`, `gemini-3-flash-preview`, `gemini-3.1-pro-preview` (01-2026), `gemini-3.1-flash-lite`, `gemini-3.5-flash`, `gemini-3.5-flash-lite`, `gemini-3.6-flash`, `gemini-3.7-flash`, `gemini-3.8-flash`, `gemini-4-argon`, `gemini-4-carbon`, the aliases `gemini-flash-latest`, `gemini-flash-lite-latest`, `gemini-pro-latest`, and others (image, audio, live, omni). **Candidates:** Flash-class `gemini-3.8-flash` (the newest Flash; Google routes 2.5 Pro to it from 2026-12-01); Pro-class `gemini-3.1-pro-preview` (the newest Pro, a preview). `gemini-4-argon` ("our efficient Gemini 4 frontier reasoning model") is a third option for CP 1.

**Prices** (Google's pricing page, `aistudio.google.com/docs/pricing`, read 2026-11-11; paid tier, standard; USD per 1M tokens; output includes thinking tokens):

| Model | Input | Output (incl. thinking) |
|---|---|---|
| Gemini 3.8 Flash | 0.75 to 2026-12-31, **1.50 from 2027-01-01** | 3.75 to 2026-12-31, **7.50 from 2027-01-01** |
| Gemini 3.6 Flash | the same as 3.8 Flash | the same |
| Gemini 3.5 Flash-Lite | 0.30 | 2.50 |
| Gemini 3 Flash Preview | 0.50 | 3.00 |
| Gemini 3.1 Pro Preview | 2.00 (≤ 200k prompt) | 12.00 (≤ 200k) |
| Gemini 4 Argon, Gemini 4 Carbon | 2.00 to 2026-11-30, 4.00 from 2026-12-01 | 10.00, then 20.00 |
| Gemini 2.5 Pro / 2.5 Flash | 1.25 / 0.30 | 10.00 / 2.50 (both routed to newer models from 2026-12-01) |

Thinking tokens are billed as output ("Output price (including thinking tokens)"; "every token the model processes is counted and billed, including thinking tokens"). Gemini 3.7 Flash has no row on the page.

**Terms** (`aistudio.google.com/docs/terms`, effective 2026-03-23; `docs/usage-policies` and `docs/logs-policy`, read 2026-11-11):

- **Paid Services:** "Google doesn't use your prompts ... or responses to improve our products." "For Paid Services, Google logs prompts and responses for a limited period of time, solely for detecting and preventing violations." Paid applies when the account "has access to a Cloud Project with an associated and active Cloud Billing account."
- **Unpaid Services:** Google "uses the content you submit to the Services and any generated responses to provide, improve, and develop Google products and services"; "human reviewers may read, annotate, and process your API input and output."
- **Abuse monitoring:** "Google retains the following data for fifty-five (55) days for the purposes of detecting and preventing violations of the Prohibited Use Policy": prompts, contextual information and output. "Data can be accessed for human review only by authorized Google employees." No opt-out or zero data retention is documented.
- **Developer logs** (separate from abuse monitoring): on by default "for Generate Content API calls in projects created on or after October 26, 2026", kept up to 55 days, not used for product improvement by default; "As a project owner, you can turn logging on or off at any time from the Logs and Datasets page in AI Studio."

**Call 1** (`a01`, the `@v1` system prompt, user message and output schema; `gemini-3.8-flash`; temperature 0.2; `responseMimeType: application/json`; the canonical schema as `responseJsonSchema`): **HTTP 200 in 22.7 s**; `modelVersion: gemini-3.8-flash`; `finishReason: STOP`; usage: prompt 4,782, output 937, **thinking 1,557** tokens; one part, `text` with a `thoughtSignature`; cost at the 2026 price about 1.3 cents (2.6 cents from 2027).

- **The schema was accepted whole** (`$schema`, `oneOf`, `pattern`, `additionalProperties`, `const`), but **`const` was not honoured:** the answer held `"format": {}` and `"catalog": {}`. Every other field followed the schema, including each item's nested `params: { kind, params }`.
- With `format` and `catalog` set by the server (as `source` is), the answer **parses into `EditPlan` and `validate` keeps all 7 items, dropping none:** KeywordPop "quit", NumberReveal "3000 hours", KeywordPop "brutal", FromTo "90 minutes to 10 minutes", ListReveal of the three reasons, NumberReveal "$12,000", KeywordPop "matter".
- **Thinking is on by default** and is most of the time and cost: 22.7 s is over D-498's hard line of 20 s.

**Call 2** (the same request; `gemini-3.1-pro-preview`): **HTTP 429**, "You exceeded your current quota", naming only free-tier metrics (`generate_content_free_tier_input_token_count`, `generate_content_free_tier_requests`, each `limit: 0` for `gemini-3.1-pro`). **So the key's project is on the free tier, not billed** (the response header `x-gemini-service-tier: standard` is the speed tier, not the billing tier). On the free tier the terms above allow Google to use what is sent to improve its products, with human review. Only `a01`, public in the repository, was sent. **Stop and ask: no further Gemini call until the founder confirms billing on the key's project.**

**Not read yet, waiting on billing:** the Pro-class answer; Flash with less thinking (its latency and plan quality); Gemini 4 Argon if the founder wants it at CP 1. Two to three calls, before P188.

Recorded, redacted shapes for P190 (request with the transcript texts cut to their lengths; full responses, which quote only `a01`; response headers without anything account-like) are in the session's scratchpad (`gemini-c1-flash-*.json`, `gemini-c2-pro-*.json`); P190 rebuilds them as `server/tests/fixtures/gemini/` with no key or account identifier.

## TE-30 - Images drawn the same in the browser and natively

Not read yet (P197).

**P183's first read (2026-11-11, d1, WARP "Microsoft Basic Render Driver"):** Vello 0.11's `Scene::draw_image(&ImageBrush, Affine)` with peniko 0.6's `ImageData { format: Rgba8, alpha_type: ImageAlphaType::Alpha }` (straight alpha accepted as is), `ImageBrush::with_quality(ImageQuality::Medium)` (bilinear) and `with_alpha(opacity)`, drawn in the overlay through `Renderer::render_to_image` with temporary arms in `vello_backend.rs` and `composite.rs` (restored byte for byte after): a 160 px disc with a 6 px soft alpha edge over flat grey at 1×, 0.5×, 1.5×, at opacity 0.5, and across the canvas edge. Centres exactly `[230, 120, 40]` at every scale; at 0.5 exactly `[179, 124, 84]`; the background `[128, 128, 128]`; across 15,120 edge samples no channel below the darker of the two colours (no dark fringe), 10,092 samples between them (soft edges); the one across the edge clipped cleanly.

## TE-31 - The drift's smoothness

**P183's first look (2026-11-11, d1, WARP):** the synthetic source of `golden_frames.rs` (1080 × 1920) drawn at 540 × 960 through the real video pass with `src` scaled linearly from 1.00 to 1.05 over 150 frames; frames 70-80: the red disc's centre moved 0.021 to 0.039 output pixels every frame (0.030 expected), never zero and never a jump. **The video pass samples a fractional crop continuously: nothing snaps the crop to whole pixels**, so a slow drift cannot step. At the canvas edge the linear drift moves about 0.09 output pixels a frame; the `STANDARD` curve's peak about doubles that. P199 reads the curve's frames.

## E-11 - Instrumentation

Not read yet. P193 and P211.
