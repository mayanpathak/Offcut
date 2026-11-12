// V6.2 (P194): which line the plan's toasts show for each reason (§8.2, §8.3,
// D-494). The words are drafts for checkpoint 3; this pins the mapping only.

import { describe, expect, it } from "vitest";

import { messages } from "./messages";

const plan = messages.plan;

describe("the plan's lines", () => {
  it("a fallback says why; invalid, timeout and catalog read as unavailable", () => {
    expect(plan.fallbackLine("signed_out")).toBe(plan.fallback.signedOut);
    expect(plan.fallbackLine("quota")).toBe(plan.fallback.quota);
    expect(plan.fallbackLine("offline")).toBe(plan.fallback.offline);
    for (const why of ["unavailable", "invalid", "timeout", "catalog"]) {
      expect(plan.fallbackLine(why)).toBe(plan.fallback.unavailable);
    }
  });

  it("a regeneration that changed nothing says why where a person can act, and that the edit is unchanged", () => {
    expect(plan.regenerateLine("signed_out")).toBe(plan.regenerateWhy.signed_out);
    expect(plan.regenerateLine("quota")).toBe(plan.regenerateWhy.quota);
    expect(plan.regenerateLine("offline")).toBe(plan.regenerateWhy.offline);
    for (const why of ["unavailable", "invalid", "timeout", "catalog", "busy", "failed", "off"]) {
      expect(plan.regenerateLine(why)).toBe(plan.regenerateFailed);
    }
    for (const line of [...Object.values(plan.regenerateWhy), plan.regenerateFailed]) {
      expect(line.endsWith("Your edit is unchanged.")).toBe(true);
    }
  });

  it("the settings line names the provider and where it is, and promises nothing about keeping (open item 35)", () => {
    for (const text of [plan.settings.help, plan.settings.row.leaves]) {
      expect(text).toContain("DeepSeek");
      expect(text).toContain("China");
      expect(text).not.toMatch(/not kept|never kept|train/i);
    }
    expect(plan.settings.help).not.toContain("Gemini");
  });
});
