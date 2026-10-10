// Prints the facts about the free tiers that Offcut is built on. None of them
// is ours to keep true: each has to be read again before launch, and whenever
// its experiment is run (TS §37, v1implementation §14).
//
// `checked` is the date the fact was last read at its source, or null if the
// experiment has not been run yet. Update it when you write the outcome into
// docs/v1/experiments.md.
//
// Usage: node scripts/check-external-facts.mjs

const FACTS = [
  {
    id: "TE-5",
    host: "Vercel (free plan)",
    fact: "The plan's terms restrict it to non-commercial use; COOP and COEP headers and the /api/v1 rewrite work; a 60 s upstream delay survives the proxy; the number of X-Forwarded-For hops is known.",
    checked: null,
    // The terms alone were read when the technical spec was written (TS §33).
    note: "terms read 2026-10 for the spec; the deployment is not measured yet",
  },
  {
    id: "TE-6",
    host: "Postgres provider (free tier)",
    fact: "A database does not expire; it accepts 5 connections; a reconnect after a week idle takes under 2 s; the storage quota is at least 5 times the 72 MB estimate.",
    checked: null,
  },
  {
    id: "TE-7",
    host: "Asset host (free tier)",
    fact: "Egress is free; HTTP Range requests answer 206; CORS is readable under COEP; a file of 150 MB is accepted; no payment card is required.",
    checked: null,
  },
  {
    id: "TE-8",
    host: "Email provider (free tier)",
    fact: "The quota is at least 100 emails a day; a sender can be verified without a purchased domain; at least 95% of links arrive in the inbox within 30 s.",
    checked: null,
    note: "due in V6",
  },
  {
    id: "TE-9",
    host: "Merchant of record",
    fact: "An India-based seller is onboarded within 2 weeks; hosted checkout carries passthrough data; there is a customer portal; a timed-out webhook is retried.",
    checked: null,
    note: "onboarding starts in V1; the rest is due in V6",
  },
  {
    id: "TE-10",
    host: "GitHub Actions (free minutes)",
    fact: "The media suites run headless on a hosted Windows runner, within the monthly free minutes.",
    // Read on 2026-10-10, in four runs of the job (docs/v2/experiments.md).
    // The minutes: a public repository pays none on a standard hosted runner,
    // and windows-latest is one. The suites: the runner has no graphics card;
    // with two arguments Chrome runs the app there, and a clip then takes
    // more than 20 minutes. So 8 of the 30 cases run there, the ones that
    // need no processed clip, and all 30 on the development machine before a
    // merge (v2implementation D-72).
    checked: "2026-10-10",
    note: "true for 8 of the 30 cases; the others need a graphics processor (D-72)",
  },
  {
    id: "TE-11",
    host: "Render (free web service)",
    fact: "A service sleeps after 15 minutes without traffic and takes about a minute to wake; one always-on service fits the 750 free instance hours a month; memory stays under 400 MB of the 512 MB.",
    checked: null,
    // TS §22.9 states these as external facts verified when the spec was written.
    note: "terms read 2026-10 for the spec; the deployment is not measured yet",
  },
];

console.log("External facts to re-verify before launch (TS §37), one line each:");
for (const { id, host, fact, checked, note } of FACTS) {
  const when = checked ?? "not checked yet";
  console.log(`${id.padEnd(5)} | last checked: ${when}${note ? ` (${note})` : ""} | ${host}: ${fact}`);
}
const open = FACTS.filter((entry) => entry.checked === null).map((entry) => entry.id);
console.log(open.length === 0 ? "All facts have a date." : `Without a date: ${open.join(", ")}.`);
