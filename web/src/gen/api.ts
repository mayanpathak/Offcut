// Generated from crates/offcut-api-types by `sh scripts/gen-types.sh`. Do not edit.
// To change anything here, change the Rust source and run the script again.

import type {
  AnonId,
  ErrorCode,
  EventKind,
  ExportId,
  FailureStage,
  Orientation,
  PipelineStage,
  Plan,
  RejectReason,
  StyleId,
  UnixSecs,
  UnsupportedReason,
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

export type HeroVariant = "outcome" | "privacy";

export type CheckResult = "pass" | "fail";

export type GpuVendor = "intel" | "amd" | "nvidia" | "apple" | "qualcomm" | "other" | "unknown";

export type MemoryBucket = "lt4" | "gb4" | "gb8plus" | "unknown";

export type Platform = "windows" | "macos" | "linux" | "chromeos" | "android" | "other" | "unknown";

export type DurationBucket = "lt30" | "lt60" | "lte90";

export type ClipSource = "user" | "sample";

export type DownloadOutcome = "ok" | "failed" | "hash_mismatch";

export type AsrBackend = "webgpu" | "wasm";

export type ReviewActionKind = "word_edit" | "event_toggle" | "style" | "crop_offset";

export type ExportedAs = "free" | "creator";

export type PostExportAnswer = "yes" | "small_edits" | "no";

export type PromptPlacement = "after_export" | "limit_reached";

export type PromptAction = "shown" | "clicked" | "dismissed";

export type SigninStep = "link_requested" | "completed";

export type CheckoutStep = "started" | "returned_success" | "returned_cancel";

/**
 * One variant per row of TS §22.6. A variant without props serializes as
 * `{"name":"preview_played"}`, with no `props` key.
 */
export type AnalyticsEvent = { "name": "landing_view", "props": { hero_variant: HeroVariant, } } | { "name": "capability_check", "props": { result: CheckResult, unsupported_reason?: UnsupportedReason | null, gpu_vendor: GpuVendor, memory_bucket: MemoryBucket, platform: Platform, } } | { "name": "clip_accepted", "props": { duration_bucket: DurationBucket, orientation: Orientation, source: ClipSource, } } | { "name": "clip_rejected", "props": { reject_reason: RejectReason, } } | { "name": "model_download", "props": { outcome: DownloadOutcome, duration_ms: number, resumed: boolean, } } | { "name": "stage_timing", "props": { stage: PipelineStage, duration_ms: number, asr_backend?: AsrBackend | null, } } | { "name": "pipeline_done", "props": { total_ms: number, n_number: number, n_list: number, n_from_to: number, n_keyword: number, } } | { "name": "preview_played" } | { "name": "review_action", "props": { action: ReviewActionKind, event_kind?: EventKind | null, enabled?: boolean | null, } } | { "name": "export_started", "props": { profile: ExportedAs, } } | { "name": "export_done", "props": { total_ms: number, profile: ExportedAs, word_edits: number, events_kept: number, events_disabled: number, style: StyleId, crop_adjusted: boolean, from_cache: boolean, } } | { "name": "export_failed", "props": { error_code: ErrorCode, stage: FailureStage, } } | { "name": "job_cancelled", "props": { stage: FailureStage, } } | { "name": "post_export_answer", "props": { answer: PostExportAnswer, } } | { "name": "upgrade_prompt", "props": { placement: PromptPlacement, action: PromptAction, } } | { "name": "signin_step", "props": { step: SigninStep, } } | { "name": "checkout_step", "props": { step: CheckoutStep, } } | { "name": "client_error", "props": { error_code: ErrorCode, stage: FailureStage, } } | { "name": "local_data_cleared" };

export type EventsBatch = { anon_id: AnonId, events: Array<AnalyticsEvent>, };

export type ApiError = { code: ApiErrorCode, retry_after_secs: number | null, };

/**
 * V1 produces `bad_request`, `not_found`, `unsupported_media_type`,
 * `payload_too_large`, `rate_limited` and `internal`. The other seven are
 * defined now so V6 adds handlers, not enum members (D-3).
 */
export type ApiErrorCode = "bad_request" | "unauthorized" | "forbidden" | "not_found" | "unsupported_media_type" | "payload_too_large" | "rate_limited" | "link_invalid" | "link_expired" | "session_expired" | "email_unavailable" | "billing_unavailable" | "internal";

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
] as const;

export const MAX_EVENTS_PER_BATCH = 50;
