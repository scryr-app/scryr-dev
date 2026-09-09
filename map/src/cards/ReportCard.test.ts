import { describe, expect, it } from "vitest";
import { type OperationalReport, reportLines } from "./ReportCard";

const report = (data: OperationalReport["data"]): OperationalReport => ({
	source: "fixture",
	scope: "unit",
	observedAt: "2026-09-08T10:00:00Z",
	runId: "1",
	attempt: 1,
	data,
});
describe("operational report summaries", () => {
	it("keeps errors and skips separate from test failures", () => {
		expect(
			reportLines(
				report({
					kind: "tests",
					passing: 15,
					failing: 1,
					errors: 2,
					skipped: 3,
					duration: 1.25,
				}),
			),
		).toEqual(["15 passed · 1 failed", "2 errors · 3 skipped", "1.25 seconds"]);
	});
	it("does not claim zero-line coverage is passing", () => {
		expect(
			reportLines(report({ kind: "coverage", covered: 0, total: 0 }))[0],
		).toContain("unavailable");
	});
	it("counts open alerts only and does not infer inventory", () => {
		const lines = reportLines(
			report({
				kind: "dependencies",
				alerts: [
					{ state: "open", severity: "high", package: "a" },
					{ state: "fixed", severity: "critical", package: "b" },
				],
			}),
		);
		expect(lines).toContain("1 open security alerts");
		expect(lines).toContain("critical: 0 · high: 1");
		expect(lines).toContain("Inventory and outdated counts not reported");
	});
});
