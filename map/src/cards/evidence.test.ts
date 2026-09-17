import { expect, it } from "vitest";
import {
	collectCommand,
	collectorStatus,
	detectedLicenses,
	resultSummary,
	safeHttpUrl,
	statusTone,
} from "./evidence";
import { evidence, results } from "./evidenceFixtures";

it("shows representative results in all six cards without merging suites or tools", () => {
	expect(resultSummary(results.git).join(" ")).toContain("feature/cards");
	expect(resultSummary(results.workflows).join(" ")).toContain(
		"failure · main",
	);
	expect(resultSummary(results.check).join(" ")).toContain("Ruff: Failed");
	expect(resultSummary(results.tests).join(" ")).toContain(
		"48 passed · 1 failed",
	);
	expect(resultSummary(results.inventory).join(" ")).toContain(
		"1 packages with unknown licenses",
	);
	expect(resultSummary(results.metrics).join(" ")).toContain("0 requests/s");
	expect(resultSummary(results.benchmark).join(" ")).toContain(
		"-25% vs baseline",
	);
});
it("keeps lifecycle separate from failing findings, stale status, and outdated inputs", () => {
	const failedTests = evidence();
	expect(collectorStatus(failedTests)).toBe("Collected");
	expect(statusTone(failedTests)).toBe("neutral");
	expect(
		collectorStatus(
			evidence(null, { state: "MISSING_TOOL" as typeof failedTests.state }),
		),
	).toBe("Missing tool");
	expect(collectorStatus(evidence(results.tests, { stale: true }))).toContain(
		"Stale",
	);
	expect(
		collectorStatus(evidence(results.tests, { stale: true, outdated: true })),
	).toContain("Outdated inputs");
});
it("never calls incomplete scans or zero coverage clean", () => {
	expect(
		resultSummary({
			...results.vulnerabilities,
			items: [],
			complete: false,
		}).join(" "),
	).toContain("Security status unknown");
	expect(
		resultSummary({ ...results.coverage, covered: 0, total: 0 }).join(" "),
	).toContain("unavailable");
});
it("restricts report links and shell-quotes CLI selector values", () => {
	expect(safeHttpUrl("javascript:alert(1)")).toBeUndefined();
	expect(safeHttpUrl("https://example.com/run")).toBe(
		"https://example.com/run",
	);
	expect(collectCommand(evidence(null, { collectorId: "unit'quoted" }))).toBe(
		"scryr collect run --manifest 'services/api' --section tests --collector 'unit'\\''quoted'",
	);
});

it("keeps scanner placeholder licenses visible as unknown evidence", () => {
	expect(detectedLicenses(["NOASSERTION", "NONE", "unknown"])).toEqual([]);
});
it("reads dependency summary counts without downloading detail arrays", () => {
	expect(
		resultSummary({ ...results.inventory, packages: [], relationships: [] })[0],
	).toBe("2 packages");
	expect(resultSummary({ ...results.licenses, items: [] })[0]).toBe(
		"0 denied · 1 need review",
	);
	expect(resultSummary({ ...results.vulnerabilities, items: [] })[0]).toBe(
		"1 advisory/package findings",
	);
});
