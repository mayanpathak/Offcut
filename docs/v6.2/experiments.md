# V6.2 experiments

The readings V6.2 takes (`v6.2implementation.md` §15). Each is dated, labelled `d1` where it was read on the founder's machine, and settles the decisions it names with an "As read" note in the plan. Nothing here holds a key, a transcript's text beyond what the repository already holds, or a recording.

## TE-15 - The plan's provider (Gemini read first; DeepSeek from P183b, D-500)

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

### The founder's decision: DeepSeek only (P183b, 2026-11-11, d1)

The founder's paid Gemini account is blocked, so **DeepSeek is the only provider, with thinking always off** (D-500). The Gemini reads above stay as history. DeepSeek's key is `PLAN_API_KEY2` (D-465).

**Models** (`GET /models`, HTTP 200): `deepseek-flash` ("DeepSeek-V4.1-Flash", per the pricing page `-1017`) and `deepseek-v4-pro` ("DeepSeek-V4-Pro", `-0813`), each with a 1M-token context. **Candidates for TE-15:** both, thinking off.

**Prices** (`api-docs.deepseek.com/quick_start/pricing`, read 2026-11-11; no date on the page; USD per 1M tokens at peak; off-peak is half; peak is 01:00-04:00 and 06:00-10:00 UTC on weekdays, excluding Chinese public holidays). The table was read as text, so the founder confirms it at CP 1:

| Model | Input, cache hit | Input, cache miss | Output |
|---|---|---|---|
| `deepseek-flash` | 0.006 | 0.30 | 1.20 |
| `deepseek-v4-pro` | 0.044 | 1.32 | 3.96 |

Concurrency limits are 2,500 (Flash) and 500 (Pro) per account. "Product prices may vary." Whether reasoning tokens bill as output is not stated; with thinking off there are none.

**API** (`guides/json_mode`, `guides/thinking_mode`, `quick_start/rate_limit`): thinking is on by default at effort `high`; `{"thinking": {"type": "disabled"}}` turns it off. `reasoning_effort` and `effort: "low"` were sent and changed nothing measurable (reasoning tokens did not fall). Thinking mode ignores `temperature`. JSON output is `{"type": "json_object"}` only: the prompt must contain the word "json" and should show the format. "The returned `content` may contain only spaces ... even when `finish_reason` is `stop`." A non-streaming request is padded with blank lines while it waits; a request not started within 10 minutes is closed; a request over the concurrency limit gets 429.

**Terms** (privacy policy updated 2026-02-10; Open Platform terms effective 2026-04-29). The API terms say nothing about training on, keeping or locating API inputs. The privacy policy says: "we directly collect, process and store your Personal Data in People's Republic of China"; personal data is used "to train and improve our technology, such as our machine learning models", with "the right to opt-out of using your Personal Data for training our models"; it is kept "for as long as you have an account". It does not cover "Personal Data collected from end users when accessing downstream systems or applications developed by developers using our open platform services". No retention period, zero-retention option or human-review rule is stated. What the product may say follows in D-492's note.

**Calls of P183b** (`a01` only, public in the repository; the `@v1` system prompt with the `@v1` output schema appended as text, the `@v1` user message; `thinking` disabled, `json_object`, temperature 0.2, `max_tokens` 8000; each answer given `format`, `catalog` and `source` by a scratch validator as the server will, then `validate` with `PlanConfig::default()`):

| Call | Model | Status, time | Tokens: in (cache hit) / out | Items | `validate` kept | Dropped |
|---|---|---|---|---|---|---|
| 1 | `deepseek-flash` | 200, 2,590 ms | 5,666 (5,504) / 460 | 8 | 7 | 1 NumberReveal, `AnchorRule` |
| 2 | `deepseek-flash` | 200, 2,805 ms | 5,666 (5,504) / 520 | 9 | 9 | 0 |
| 3 | `deepseek-flash` | 200, 2,483 ms | 5,666 (5,504) / 520 | 9 | 7 | 1 NumberReveal `NumberUnresolved`, 1 FromTo `AnchorRule` |
| 4 | `deepseek-v4-pro` | 200, 5,063 ms | 5,663 (0) / 618 | 5 | 5 | 0 |
| 5 | `deepseek-flash`, `json_schema` strict | **400** in 312 ms, "This response_format type is unavailable now" | - | - | - | - |

- **Every answer parsed into `EditPlan`.** Unlike Gemini, DeepSeek wrote `format: 1` and `catalog: 1` itself. It left out `source` and wrote `cuts`, `chapters` and `music` empty and `extras` `{}`. The server still sets `format`, `catalog` and `source` (D-460).
- **What it chose.** Flash: the three numbers ("3000 hours", "$12,000", and "8" from "8 hours", which `validate` dropped once), "90 minutes to 10 minutes", the three reasons, and 3-4 keyword pops ("brutal", "luck", "timeline", "rough", "matter"). Pro: the numbers, the from-to and the list, with no keyword pop. The same request gave different plans across runs; plans are stored, never re-requested (D-469).
- **Against TE-15's pass lines (first read, one transcript):** parse 4/4; latency far under D-498's 10 s; **Flash's median drop rate is 12.5% of items, over the 10% line** (two `AnchorRule` drops on the anchor's length or alignment, one `NumberUnresolved`), Pro's is 0%. P186's prompt prose and TE-15's 15 transcripts read it properly.
- **Cost per plan** at peak prices: Flash about $0.0007 with the cached system prompt, $0.0023 without; Pro about $0.010 (no cache hit yet). Off-peak is half.
- **Other DeepSeek calls on 2026-11-11, before P183b** (the founder's request, outside a prompt): one JSON ping with no transcript; then on `a01` with thinking on: Flash 35 s / 6,000 reasoning tokens, Pro 145 s / 12,000, with all their plans valid; and Flash with thinking off once, 3.5 s, 9 items kept. The account's balance went from $1.99 to $1.93 over all thirteen calls.

Recorded, redacted shapes for P190 (each request with message texts cut to their lengths and the schema named, not copied; each response with `id` and `system_fingerprint` redacted; response headers without ids, cookies or tracing) are in the session's scratchpad (`p183ds/*.record.json`); P190 rebuilds them as `server/tests/fixtures/deepseek/` with no key or account identifier.

### The full read (P188, 2026-11-11 23:15-23:18 UTC, d1; the founder cut it to 30 plans)

**What was sent.** The 15 transcripts of D-493 (`a01`, `w01`-`w14`, all written scripts in the repository; never `a02`), each once to `deepseek-flash` and once to `deepseek-v4-pro`: 30 plans, **33 calls** (3 retries, below). D-500's request exactly: the `@v2` system prompt, then "Answer with one JSON object that matches this JSON Schema exactly:" and the compact `@v2` output schema (15,194 characters, identical for every transcript); the `@v2` user message, built from each file as `validate_rules_plans.rs` builds it (stressed words at 2.0); `thinking: {"type": "disabled"}`, `json_object`, temperature 0.2, `max_tokens` 8000. Each answer was given `format`, `catalog` and `source` as the server will, parsed into `EditPlan` and validated with the file's edits and `PlanConfig::default()` by a scratch tool built on the repository's crates (not committed). Off-peak; the account's balance went from $1.93 to $1.89.

| | `deepseek-flash` | `deepseek-v4-pro` | Pass line |
|---|---|---|---|
| Parse, first try / after one retry | 13 / 15, **15 / 15** | 14 / 15, **15 / 15** | ≥ 98% after one retry: **both pass** |
| Items written / kept | 128 / 104 | 116 / 69 | |
| Items per plan, median (kept) | 8 (7) | 7 (5) | |
| **Item drop rate, median (mean)** | **16.7%** (17.8%) | **28.6%** (36.2%) | ≤ 10%: **both fail** |
| Latency p50 / p90 / max | 2.8 / **3.3** / 6.0 s | 4.2 / **6.1** / 7.4 s | p90 ≤ 10 s: **both pass** |
| Tokens in (cache hit) / out, median | 5,151 (3,840) / 549 | 5,148 (3,968) / 813 | |
| Thinking tokens | 0 | 0 | |
| Cost per plan at peak, median (no cache) | $0.0011 ($0.0022) | $0.0050 ($0.0100) | recorded |
| Hook / caption / hashtags written | 15 / 15 / median 5 | 15 / 15 / median 8 | |
| Hook titles kept / written | 1 / 1 | 7 / 10 | |

**Drops by kind and reason.**

- **Flash** (24 items): `punch_in` too close 7, anchor rule 5; `icon_pop` zone taken 4; `number_reveal` number unresolved 3, anchor rule 1; `emoji` density 2; `from_to` too close 1, number unresolved 1. One hook was too long.
- **Pro** (47 items): `emphasis` anchor rule 14; `punch_in` too close 9, anchor rule 5; `number_reveal` anchor rule 7, number unresolved 1; `icon_pop` anchor rule 3; `emoji` anchor rule 2, density 1; `hook_title` params mismatch 2, hook missing 1; `from_to` too close 1, number unresolved 1. Three hooks were too long.

**What the plans show** (all 30 annotated in `~/v6.2-review/plans/`, outside git; ten to read first named in its README):

- **Pro anchors every item to its whole sentence.** An emphasis or icon meant for one word comes with a 9-word range, and a number with 11 words, so `AnchorRule` takes most of its items. Its choices inside those sentences are sensible.
- **Flash follows the anchor rules, but zooms on almost every sentence** (56 zooms in 15 short scripts, usually `push` across the whole sentence). Zooms in back-to-back sentences then break the kind's 1.5 s gap. It also puts icons on top of numbers in the event zone.
- **Each parse failure was one name outside the 72** (the emoji `cat`, `lock` used as an emoji, the icon `cloud`). Serde refuses the whole plan for one unknown name; the one retry rescued each of the three plans.
- **A number range that is not exactly the number** (`NumberUnresolved`, 5 in all), as in P183b.
- Both models wrote a hook, a post caption and hashtags every time. No hashtag was dropped. Four hooks were dropped as too long (over 48 characters or 9 words).

**Recommendation for CP 1 (D-498):** `deepseek-flash`, thinking off. It is clearly better here, not just cheaper: 104 of 128 items kept against Pro's 69 of 116, p90 3.3 s against 6.1 s, and a fifth of the cost. Both miss the 10% drop line, and the causes are in the prompt, not the model. Offered at CP 1:

1. **One prompt revision, then TE-15 on Flash again** (15 calls, about $0.02; a new `@v2` read, since nothing ships on it yet). It would say how to write a one-word anchor (`{"start": w, "end": w + 1}`), that a zoom covers only its own words and not every sentence needs one, and that an icon never shares the event zone with a number. **Recommended.**
2. **Item-level parsing in P190:** the adapter parses each item alone, so an unknown name drops that item (`ParamsMismatch`) rather than the plan. This saves a retry, about 7% of calls here. **Recommended,** as P190's design, with the founder's yes.
3. Accept Flash as it is: `validate` is the gate, and drops cost nothing but a fuller plan.
4. Pro: not recommended.

### Checkpoint 1 and the prompt revision (P188b, 2026-11-12 07:16-07:17 UTC, d1)

**The founder's answers (CP 1, 2026-11-12):** `deepseek-flash`, thinking off; the budget prices Flash 0.30 in / 1.20 out USD per 1M tokens (peak, cache miss; D-463 counts every input token at the miss price); yes to the one prompt revision and the 15-call Flash re-read; yes to item-level parsing in P190.

**The revision** (`@v2` read again: nothing ships on it yet). The "finished short" paragraph says to zoom only on the sentences that land hardest, so most have none (it said "zoom in on each sentence's key claim"). The zoom paragraph says "about one sentence in three", that a zoom covers only its claim's words, and that a zoom starts at least 1.5 s after the last one ends, so two sentences in a row are never both zoomed. A new paragraph covers pointing at words: a one-word item is anchored `{"start": w, "end": w + 1}`, never to its sentence; a number's range is exactly the number and its unit, with no word before it (`$2,400`, not `cost $2,400`); a `number_reveal` is anchored to those words or to at most 6 around them. A second new paragraph covers the event zone: `number_reveal`, `list_reveal`, `from_to` and `icon_pop` share it, each holding it until 1.5 s after its last word, so an icon never goes on a number or just after one. `punch_in`'s purpose says `push` covers "the few words that build to it" (it said "a sentence that builds"). Every number in these paragraphs comes from the catalog or `validate`'s constants. The system message grew from 15,194 to 16,159 characters.

**The re-read.** The same harness as P188, with the same request (D-500), the same 15 transcripts and `deepseek-flash` only: 15 calls, all HTTP 200, `finish_reason: stop`, no thinking tokens. **Peak hours** (07:16 UTC on a weekday): about $0.017 for the 15 calls at peak prices. The account's balance, shown to the cent, read $1.87 before and after.

| `deepseek-flash` | P188 (`@v2` as P186 wrote it) | **P188b (revised)** | Pass line |
|---|---|---|---|
| Parse, first try / after one retry | 13 / 15, 15 / 15 | **15 / 15**, 15 / 15 | ≥ 98% after one retry: pass |
| Calls | 17 | 15 | |
| Items written / kept | 128 / 104 | 111 / 98 | |
| Items per plan, median (kept) | 8 (7) | 7 (6) | |
| **Item drop rate, median (mean)** | 16.7% (17.8%) | **14.3% (10.9%)** | ≤ 10%: **still fails on the median** |
| Plans at or under 10% drops | 4 of 15 | 6 of 15 | |
| Zooms written / dropped | 56 / 12 | **45 / 1** | |
| Latency p50 / p90 / max | 2.8 / 3.3 / 6.0 s | 2.8 / **3.5** / 3.6 s | p90 ≤ 10 s: pass |
| Tokens in (cache hit) / out, median | 5,151 (3,840) / 549 | 5,408 (4,096) / 500 | |
| Cost per plan at peak, median (no cache) | $0.0011 ($0.0022) | $0.0011 ($0.0023) | recorded |
| Hook / caption / hashtags written | 15 / 15 / median 5 | 15 / 15 / median 5 | |

**Drops (13 items).** `from_to` number unresolved 5; `number_reveal` number unresolved 3; `icon_pop` zone taken 3, density 1; `punch_in` too close 1. One hook was too long. **No `AnchorRule` drop is left** (P188: 6), and no emoji or icon name fell outside the 72.

**What is left: the number's last word.** Seven of the eight unresolved numbers take one word past the number: `$12 a` (month), `$2,800 a`, `$3,000 a`, `$60 an` (hour), `3 sets of`, `10 pm` (`pm` is not a unit `parse_quantity` reads), and one takes a word before it (`Rust 1.80`); one from-to's second side is not a number at all (`twice as fast`). The zooms are now 2 to 4 a script (`a01`: 3 in 17 sentences), each on a few words. The annotated plans are in `~/v6.2-review/p188b/plans/`.

**Read.** The revision did what it was for: zoom drops 12 → 1, anchor drops 6 → 0, parse failures 2 → 0, mean drops 17.8% → 10.9%. The median line still fails at 14.3%, now on the number's range alone. Not acted on (the founder approved one revision): a second line in the pointing paragraph ("the range ends on the number or its unit word: never `a`, `an`, `of`, `per`"), or letting `validate` trim a trailing filler word, would take most of the rest. Item-level parsing (P190) does not change these numbers: no plan failed to parse here.

### The second revision: where a number's range ends (P188c, 2026-11-12 08:03-08:04 UTC, d1)

**The founder's yes (2026-11-12)** to the second fix P188b offered. The pointing paragraph now says a number's range starts on the number and ends on the number or on the one unit word just after it (hours, percent, dollars, users), never on `a`, `an`, `of`, `per` or `pm` ("`$12`" in "`$12 a month`"), and that both sides of a from-to are numbers the speaker says. System message: 16,366 characters. The same harness, request and 15 transcripts; `deepseek-flash` only; **16 calls** (one parse retry: `w14`'s first answer used the icon `lock` as an emoji, a case P190's item-level parsing now keeps without a retry). All HTTP 200, `stop`, no thinking tokens. Peak hours; about $0.016 at peak prices; the balance read $1.85 before and after.

| `deepseek-flash` | P188 | P188b | **P188c** | Pass line |
|---|---|---|---|---|
| Parse, first try / after one retry | 13 / 15 | 15 / 15 | 14 / 15, 15 / 15 | ≥ 98% after one retry: pass |
| Items written / kept | 128 / 104 | 111 / 98 | 103 / 94 | |
| **Item drop rate, median (mean)** | 16.7% (17.8%) | 14.3% (10.9%) | **0.0% (8.5%)** | ≤ 10%: **pass** |
| Plans with no drop / at or under 10% | 2 / 4 | 5 / 6 | 8 / 8 | |
| Zooms written / dropped | 56 / 12 | 45 / 1 | 38 / 0 | |
| Latency p50 / p90 / max | 2.8 / 3.3 / 6.0 s | 2.8 / 3.5 / 3.6 s | 2.8 / **3.3** / 3.6 s | p90 ≤ 10 s: pass |
| Cost per plan at peak, median (no cache) | $0.0011 ($0.0022) | $0.0011 ($0.0023) | $0.0010 ($0.0023) | recorded |

**Drops (9 items).** `from_to` number unresolved 5; `number_reveal` number unresolved 2; `icon_pop` zone taken 2. No extra was dropped. The from-to sides still end on `a` or `an` in three plans (`$12 a` to `$29 a`, `$2,800 a`, `$60 an`), as in P188b: the rule moved the number reveals, not the from-tos. The others: `3 sets` (`sets` is not a count noun `parse_quantity` knows), `Version 2`, and `twice` to `less`. The annotated plans are in `~/v6.2-review/p188c/plans/`.

**Read.** TE-15's lines all pass on `deepseek-flash` with thinking off: parse after one retry 15/15, median drops 0.0%, p90 3.3 s. No further revision is proposed; `validate` drops the rest, at the cost of a fuller plan only.

## TE-30 - Images drawn the same in the browser and natively

Not read yet (P197).

**P183's first read (2026-11-11, d1, WARP "Microsoft Basic Render Driver"):** Vello 0.11's `Scene::draw_image(&ImageBrush, Affine)` with peniko 0.6's `ImageData { format: Rgba8, alpha_type: ImageAlphaType::Alpha }` (straight alpha accepted as is), `ImageBrush::with_quality(ImageQuality::Medium)` (bilinear) and `with_alpha(opacity)`, drawn in the overlay through `Renderer::render_to_image` with temporary arms in `vello_backend.rs` and `composite.rs` (restored byte for byte after): a 160 px disc with a 6 px soft alpha edge over flat grey at 1×, 0.5×, 1.5×, at opacity 0.5, and across the canvas edge. Centres exactly `[230, 120, 40]` at every scale; at 0.5 exactly `[179, 124, 84]`; the background `[128, 128, 128]`; across 15,120 edge samples no channel below the darker of the two colours (no dark fringe), 10,092 samples between them (soft edges); the one across the edge clipped cleanly.

**P196's read (2026-11-12, d1, WARP), the real bundle through `Renderer::set_images` and `render_to_image` (`offcut-render/tests/image_draw.rs`):** over flat grey 128, every pixel of `laughing` drawn at 1x equals its file blended in sRGB values to within 0.50 of a level, and `heart_eyes` at opacity 0.5 within 0.74; the white `check` icon at 0.5x, 1.37x and 1.5x, each starting between pixels: no channel below 127 (no dark fringe from the bundle's 0, 0, 0, 0 transparent pixels), every pixel neutral to 1 level, 8,390 soft edge pixels and 25,680 white. `image_basic.png` (three emoji at 0.5x, 1x, 1.5x, one at 0.5 opacity, one across the right edge, a white icon, a half-dark rectangle over one emoji) written once on WARP and read at 200%. **WARP is not bit-exact between runs:** five draws of the old golden `bold_number`, with and without P196, differed from it in 0 to 2 pixels by 1 level; every old golden reads SSIM 1.0000.

## TE-31 - The drift's smoothness

**P183's first look (2026-11-11, d1, WARP):** the synthetic source of `golden_frames.rs` (1080 × 1920) drawn at 540 × 960 through the real video pass with `src` scaled linearly from 1.00 to 1.05 over 150 frames; frames 70-80: the red disc's centre moved 0.021 to 0.039 output pixels every frame (0.030 expected), never zero and never a jump. **The video pass samples a fractional crop continuously: nothing snaps the crop to whole pixels**, so a slow drift cannot step. At the canvas edge the linear drift moves about 0.09 output pixels a frame; the `STANDARD` curve's peak about doubles that. P199 reads the curve's frames.

## E-11 - Instrumentation

Not read yet. P193 and P211.
