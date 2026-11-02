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

## TE-9: the merchant. Dodo Payments, test mode, 2026-11-03

| Reading | Result |
|---|---|
| Hosted checkout with `metadata`; a customer portal | Both work, through `adapter.rs` |
| `Activated`, `Renewed`, `PastDue`, `CancelScheduled`, `Canceled`, `Refunded` | All six observed, from real deliveries |
| A timed-out webhook retried | A 500 is retried after about 6 s, with the same `webhook-id` |
| A test purchase updates the row and the next token says Creator | Through the fake provider only so far. Read on the deployed site at G 14.6 |

Approval for live mode is not part of this: it belongs to the founding release (`buildplan.md` §8A.6).
