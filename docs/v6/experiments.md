# experiments.md - outcome of every experiment V6 reads

## Render: can it send mail? (D-253, D-304)

| Reading | Result |
|---|---|
| Port probe, Render free web service, 2026-11-02 | 2525 open; 587 and 465 blocked |
| One authenticated send from a throwaway Render service, STARTTLS on 2525, 2026-11-02 17:07 UTC | `sent` in its logs |

Decision: `lettre` over SMTP on 2525 stands (D-252). D-253's stop is not met.

## TE-8: mail delivery. Cut down by the founder

**Planned (V6 section 17):** 50 mails across Gmail, Outlook and Yahoo; at least 95% in the inbox within 30 s.

**Founder's decision, 2026-11-02:** two mailboxes exist, both Gmail, nothing has gone to spam so far, and "if it works it works". The 50 mails are not sent.

| Mail | To | Result |
|---|---|---|
| Test 1, script, 2026-11-02 10:53 UTC | The sender's own Gmail | Inbox (founder) |
| Test 2, the mail probe, D1 and Render | The sender's own Gmail | The founder saw none in spam |
| The real template, through `SmtpMailer`, 16:25 UTC | The sender's own Gmail | The founder saw none in spam |
| The real template, 3 mails, 17:22 UTC | A second Gmail address | Accepted by Brevo. To be read: inbox or spam, the visible From, the link's host |

**What this does not prove.** Nothing was sent to Outlook, Yahoo or any other provider, and Brevo marks this sender "Freemail domain is not recommended" (D-256). A person on another provider may not get a sign-in link, and V6 has no other way in. The pass of BP §8.2 is not met; the founder accepted that. Read it again with real sign-ups, from the alpha on: a rise in `signin_step` `link_requested` with no `completed` is the sign. The remedy stays a domain, the founder's call.

Quota: Brevo's free plan, 300 a day against the 100 the pass asks.

## TE-9: the merchant. Dodo Payments, test mode, 2026-11-03 and 2026-11-04

| Reading | Result |
|---|---|
| Hosted checkout with `metadata`; a customer portal | Both work, through `adapter.rs` |
| `Activated`, `Renewed`, `PastDue`, `CancelScheduled`, `Canceled`, `Refunded` | All six observed, from real deliveries (P144) |
| A timed-out webhook retried | A 500 is retried after about 6 s, with the same `webhook-id`. A true timeout, from a sleeping host, is read on the deployed site at G 14.6 |
| A test purchase updates the row and the next token says Creator | **Yes, on D1, 2026-11-04 (P149).** Below |
| A partial refund | `refund.succeeded` with `is_partial: true`; ends nothing (D-277). Below |
| `subscription.plan_changed` | Real, and applied (D-301). Below |
| `subscription.past_due` | Not produced: a declined renewal sends `subscription.on_hold` (P144). Both are `PastDue` |
| Can the customer portal change the plan? | Not read. If it can, the event is handled; read at G 14.6 or whenever the founder opens the portal |

### P149: the whole purchase on D1, with Dodo's own deliveries

A local server gets no webhook, so the throwaway logger of G 8.2 (still registered in test mode) received each delivery, and a script in the session's scratchpad forwarded each one to the local API on port 8080 within seconds: the same bytes, the same `webhook-id`, `webhook-timestamp` and `webhook-signature`. So the local server verified Dodo's real signatures, inside the 5-minute window. Nothing was signed by us. Before that, a real delivery with one byte changed was refused with 401, and the exact one was accepted.

| Step | Who | Read |
|---|---|---|
| Sign in with a real mail; Free export | Founder | `signin_step` `link_requested`, `completed`; a `free` receipt; the file passes `verify_mp4.py --profile free` (720 x 1280, watermark correlation 0.78) |
| The prompt's offer, $15/month, Dodo's test card | Founder | `upgrade_prompt` `shown`, `clicked`; `checkout_step` `started` |
| Dodo's deliveries | Script | `subscription.updated`, `active`, `renewed`, `payment.succeeded` within 2 s, each retried 6-8 s later with the same id. Every subscription event carries `metadata.user_id` |
| The local row | Agent, local database | `creator`, `monthly`, `active`, period end 2026-12-04, the owner from `metadata`. Three `webhook_events` rows: a retry is a duplicate, and `payment.*` is not stored |
| The return | Founder | `checkout_step` `returned_success`, after the row was written |
| The next export | Founder | A `creator` receipt, so the token said Creator; the file passes `verify_mp4.py --profile creator` (1080 x 1920, watermark correlation 0.06) |
| Plan change, monthly to annual, `do_not_bill`, through the API | Agent | `subscription.plan_changed`, with `updated`, `renewed` and a payment of 0. The row reads `annual`, period end still 2026-12-04: Dodo keeps the cycle and charges the year then |
| Cancel, through the API | Agent | `subscription.cancelled` and `updated`. The row reads `canceled`, period end 2026-12-04: Creator until then (TS §22.5) |
| A partial refund, 5 USD of a P144 payment, through the API | Agent | The API and the delivery both say `is_partial: true`; the same 16 fields as the full refund. The local server answered 200 and changed nothing |

**What this shows.** TE-9's pass is met in test mode: all six kinds, a retried delivery with the same id, and a purchase that turns the next token Creator. What is left for G 14.6: the same purchase on the deployed site with its own endpoint, a delivery to a sleeping host, and the portal's options.

**Found.** The founder's test purchase was charged **1,476.89 INR**, not 15 USD: Dodo shows a local currency to a buyer it places in India. The pricing section names dollars. Worth reading in live mode before the founding release.

**Afterwards.** No test subscription is running: the three in test mode read `cancelled`, `cancelled`, `failed`.

Approval for live mode is not part of this: it belongs to the founding release (`buildplan.md` §8A.6). On 2026-11-04 Dodo approved the founder's identity and bank, and put the product review on hold because the live site reads as a waitlist (open item 10).
