# Per-file specification template

Copy of `technicalspec.md` §34.2. Write one specification per file from this template; §34.1 lists the context the implementer receives with it.

```text
Path:
Purpose:                 one sentence
Responsibility:          what this file owns
Non-responsibilities:    what it must never do (copy the row from §2 / §6)
Allowed dependencies:    exact module or crate paths; anything else is forbidden
Public API:              full signatures, exactly as in this spec
Types:                   new types defined here; types consumed (with their home file)
Per-function contract:   for each public function: input, output, errors, side effects, postcondition
State ownership:         state this file owns; state it reads; state it must not touch
Concurrency:             thread/worker it runs on; cancellation points; reentrancy
Performance:             budget from §30 or the owning section
Security and privacy:    data it may see; data it must not log, store or send
Constraints:             lints that apply; banned APIs; size limit (400 lines)
Definition of done:      named test files that must pass; scripts that must stay green
```
