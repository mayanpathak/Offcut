// Generated from crates/offcut-api-types by `sh scripts/gen-types.sh`. Do not edit.
// To change anything here, change the Rust source and run the script again.

import type {
  AnonId,
  DurMs,
  EditPlan,
  ErrorCode,
  EventKind,
  ExportId,
  FailureStage,
  Orientation,
  PipelineStage,
  Plan,
  RejectReason,
  StyleId,
  TimeMs,
  UnixSecs,
  UnsupportedReason,
  WordRange,
} from "./domain";

export type MagicLinkRequest = { email: string, };

export type VerifyRequest = { token: string, };

export type SessionResponse = { access_token: string, access_expires_at: UnixSecs, entitlement_token: string, };

export type MeResponse = { email: string, plan: Plan, subscription: SubscriptionSummary | null, entitlement_token: string, };

/**
 * What a waitlist signup is waiting for.
 */
export type Wanted = "launch" | "safari" | "firefox" | "mobile" | "linux";

export type NotifyMeRequest = { email: string, wanted: Wanted, };

export type AccountExport = { user: ExportedUser, sessions: Array<ExportedSession>, subscription: SubscriptionSummary | null, usage_receipts: Array<ExportedReceipt>, };

export type ExportedUser = { email: string, created_at: UnixSecs, };

/**
 * Session metadata only: no token hashes (TS §22.7).
 */
export type ExportedSession = { created_at: UnixSecs, last_used_at: UnixSecs, expires_at: UnixSecs, };

export type ExportedReceipt = { export_id: ExportId, created_at: UnixSecs, plan_at_export: Plan, };

export type SubscriptionSummary = { status: SubscriptionStatus, interval: BillingInterval, current_period_end: UnixSecs, cancel_at_period_end: boolean, };

export type SubscriptionStatus = "active" | "past_due" | "canceled";

export type BillingInterval = "monthly" | "annual";

/**
 * What can be bought (PS §11). The founding offer is the E-5 presale.
 */
export type Offer = "creator_monthly" | "creator_annual" | "creator_annual_founding";

export type CheckoutRequest = { offer: Offer, };

export type UrlResponse = { url: string, };

export type UsageReceiptRequest = { export_id: ExportId, };

export type EntitlementResponse = { entitlement_token: string, };

/**
 * One word as the provider heard it, with its written form ("year,", "3000").
 * Times are milliseconds from the start of the audio sent.
 */
export type CloudWord = { text: string, start_ms: number, end_ms: number, };

/**
 * The answer of `POST /transcribe`. The server keeps none of it.
 */
export type TranscribeResponse = { words: Array<CloudWord>,
/**
 * The duration of the audio the provider heard, in milliseconds.
 */
duration_ms: number,
/**
 * The provider and model, such as "groq:whisper-large-v3-turbo". It
 * becomes the transcript's `model_version`.
 */
model: string, };

/**
 * The body of `GET /healthz`.
 */
export type HealthResponse = { ok: boolean, version: string,
/**
 * Whether `POST /transcribe` will try. Absent from a server older than
 * the cloud transcription release, which reads as `false`.
 */
cloud_transcribe: boolean, };

/**
 * One word of the transcript, with how it was said.
 */
export type PlanWord = { text: string, start: TimeMs, end: TimeMs,
/**
 * How much louder than the speaker's average, in standard deviations.
 */
energy_z: number,
/**
 * How much higher than the speaker's average, in standard deviations.
 */
pitch_z: number, };

/**
 * The body of `POST /plan`: one clip's transcript.
 */
export type PlanRequest = {
/**
 * The client's `CATALOG_VERSION`; another than the server's is refused (409).
 */
catalog: number, duration: DurMs, words: Array<PlanWord>,
/**
 * Each sentence's words, in order.
 */
sentences: Array<WordRange>, };

/**
 * The answer of `POST /plan`: the model's plan as the server read it, not
 * yet validated (the client validates it, INV-21).
 */
export type PlanResponse = { plan: EditPlan,
/**
 * The plans left this month on a quota that blocks (Free); `None` on Creator.
 */
plans_remaining: number | null, };

export type HeroVariant = "outcome" | "privacy";

export type CheckResult = "pass" | "fail";

export type GpuVendor = "intel" | "amd" | "nvidia" | "apple" | "qualcomm" | "other" | "unknown";

export type MemoryBucket = "lt4" | "gb4" | "gb8plus" | "unknown";

export type Platform = "windows" | "macos" | "linux" | "chromeos" | "android" | "other" | "unknown";

export type DurationBucket = "lt30" | "lt60" | "lte90";

export type ClipSource = "user" | "sample";

export type DownloadOutcome = "ok" | "failed" | "hash_mismatch";

export type AsrBackend = "webgpu" | "wasm" | "cloud";

export type ReviewActionKind = "word_edit" | "event_toggle" | "style" | "crop_offset" | "caption_position" | "highlight_numbers" | "replace_all" | "undo" | "redo";

export type ExportedAs = "free" | "creator";

export type PostExportAnswer = "yes" | "small_edits" | "no";

export type PromptPlacement = "after_export" | "limit_reached";

export type PromptAction = "shown" | "clicked" | "dismissed";

export type SigninStep = "link_requested" | "completed";

export type CheckoutStep = "started" | "returned_success" | "returned_cancel";

export type CloudOutcome = "ok" | "unavailable" | "limited" | "timeout" | "offline" | "unsupported" | "failed";

/**
 * One variant per row of TS §22.6. A variant without props serializes as
 * `{"name":"preview_played"}`, with no `props` key.
 */
export type AnalyticsEvent = { "name": "landing_view", "props": { hero_variant: HeroVariant, } } | { "name": "capability_check", "props": { result: CheckResult, unsupported_reason?: UnsupportedReason | null, gpu_vendor: GpuVendor, memory_bucket: MemoryBucket, platform: Platform, } } | { "name": "clip_accepted", "props": { duration_bucket: DurationBucket, orientation: Orientation, source: ClipSource, } } | { "name": "clip_rejected", "props": { reject_reason: RejectReason, } } | { "name": "model_download", "props": { outcome: DownloadOutcome, duration_ms: number, resumed: boolean, } } | { "name": "stage_timing", "props": { stage: PipelineStage, duration_ms: number, asr_backend?: AsrBackend | null, } } | { "name": "pipeline_done", "props": { total_ms: number, n_number: number, n_list: number, n_from_to: number, n_keyword: number, } } | { "name": "preview_played" } | { "name": "review_action", "props": { action: ReviewActionKind, event_kind?: EventKind | null, enabled?: boolean | null, } } | { "name": "export_started", "props": { profile: ExportedAs, } } | { "name": "export_done", "props": { total_ms: number, profile: ExportedAs, word_edits: number, events_kept: number, events_disabled: number, style: StyleId, crop_adjusted: boolean, from_cache: boolean, } } | { "name": "export_failed", "props": { error_code: ErrorCode, stage: FailureStage, } } | { "name": "job_cancelled", "props": { stage: FailureStage, } } | { "name": "post_export_answer", "props": { answer: PostExportAnswer, } } | { "name": "upgrade_prompt", "props": { placement: PromptPlacement, action: PromptAction, } } | { "name": "signin_step", "props": { step: SigninStep, } } | { "name": "checkout_step", "props": { step: CheckoutStep, } } | { "name": "client_error", "props": { error_code: ErrorCode, stage: FailureStage, } } | { "name": "local_data_cleared" } | { "name": "asr_cloud", "props": { outcome: CloudOutcome, duration_ms: number, } };

export type EventsBatch = { anon_id: AnonId, events: Array<AnalyticsEvent>, };

export type ApiError = { code: ApiErrorCode, retry_after_secs: number | null, };

/**
 * V1 produces `bad_request`, `not_found`, `unsupported_media_type`,
 * `payload_too_large`, `rate_limited` and `internal`. The other seven are
 * defined now so V6 adds handlers, not enum members (D-3).
 */
export type ApiErrorCode = "bad_request" | "unauthorized" | "forbidden" | "not_found" | "unsupported_media_type" | "payload_too_large" | "rate_limited" | "link_invalid" | "link_expired" | "session_expired" | "email_unavailable" | "billing_unavailable" | "transcribe_unavailable" | "plan_quota" | "plan_catalog" | "plan_unavailable" | "plan_invalid" | "internal";

export const ANALYTICS_EVENT_DOCS = [
  { name: "landing_view", description: "That the landing page was opened, and which of our headlines it showed." },
  { name: "capability_check", description: "Whether this browser can run Offcut and, if not, why, with the maker of the graphics chip, a rough memory size and the operating system family." },
  { name: "clip_accepted", description: "That a clip was accepted, with its rough length, its orientation and whether it was your own clip or the sample." },
  { name: "clip_rejected", description: "That a clip could not be used, and which rule it broke." },
  { name: "model_download", description: "Whether the speech model downloaded, how long that took and whether the download was resumed." },
  { name: "stage_timing", description: "How long each processing stage took." },
  { name: "pipeline_done", description: "How long processing took in total, and how many numbers, lists, comparisons and keywords were found." },
  { name: "preview_played", description: "That the preview was played." },
  { name: "review_action", description: "That you changed something while reviewing, and what kind of change it was, never the words themselves." },
  { name: "export_started", description: "That an export started, and whether it was a free or a Creator export." },
  { name: "export_done", description: "That an export finished, with how long it took, how many words you edited, how many visual moments you kept or turned off, the caption style and whether you moved the crop." },
  { name: "export_failed", description: "That an export failed, with the error code and the stage it failed in." },
  { name: "job_cancelled", description: "That you cancelled a job, and at which stage." },
  { name: "post_export_answer", description: "Your answer to the one question we ask after an export." },
  { name: "upgrade_prompt", description: "That an upgrade prompt was shown, clicked or dismissed, and where it appeared." },
  { name: "signin_step", description: "That a sign-in link was requested or that sign-in was completed, never the email address." },
  { name: "checkout_step", description: "That checkout was started, completed or cancelled." },
  { name: "client_error", description: "That something went wrong, with the error code and the stage it happened in." },
  { name: "local_data_cleared", description: "That you cleared the data Offcut keeps in this browser." },
  { name: "asr_cloud", description: "Whether cloud transcription of a clip worked or why it fell back to this device, and how long it took, never the words." },
] as const;

export const MAX_EVENTS_PER_BATCH = 50;
