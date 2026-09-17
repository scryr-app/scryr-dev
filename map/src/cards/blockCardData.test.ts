import { describe, expect, it } from "vitest";
import { getBlockCardData } from "./blockCardData";
import { evidence, results } from "./evidenceFixtures";

describe("typed collector card data", () => {
	it("never synthesizes results from static card values or metadata", () => {
		const data = getBlockCardData({
			rawJsonString: JSON.stringify({
				metrics: { errorRate: 0 },
				tests: { passing: 99 },
				repository: { stars: 100 },
			}),
			evidence: [],
		});
		expect(Object.values(data)).toEqual([[], [], [], [], [], []]);
	});
	it("keeps configured collectors visible before their first result and preserves list order", () => {
		const data = getBlockCardData({
			rawJsonString: JSON.stringify({
				manifestId: "services/api",
				tests: [
					{ kind: "junit", id: "integration" },
					{ kind: "pytest", id: "unit" },
				],
			}),
			evidence: [evidence()],
		});
		expect(data.tests.map((item) => item.collectorId)).toEqual([
			"integration",
			"unit",
		]);
		expect(data.tests[0]).toMatchObject({ state: "WAITING", latest: null });
		expect(data.tests[1].latest?.result).toEqual(results.tests);
	});
	it("removes undeclared collectors immediately and rejects an observation from a replaced integration", () => {
		const data = getBlockCardData({
			rawJsonString: JSON.stringify({
				tests: [{ kind: "vitest", id: "unit" }],
			}),
			evidence: [
				evidence(),
				evidence(results.coverage, {
					collectorId: "coverage",
					integration: "lcov",
				}),
			],
		});
		expect(data.tests).toHaveLength(1);
		expect(data.tests[0]).toMatchObject({
			integration: "vitest",
			state: "WAITING",
			latest: null,
		});
	});
	it("keeps remote workflows in Repository and local checks in Checks", () => {
		const workflow = evidence(results.workflows, {
			section: "REPOSITORY" as ReturnType<typeof evidence>["section"],
			collectorId: "ci",
			integration: "github_actions",
		});
		const check = evidence(results.check, {
			section: "CHECKS" as ReturnType<typeof evidence>["section"],
			collectorId: "lint",
			integration: "ruff",
		});
		const data = getBlockCardData({
			rawJsonString: JSON.stringify({
				repository: [{ kind: "github_actions", id: "ci" }],
				checks: [{ kind: "ruff", id: "lint" }],
			}),
			evidence: [check, workflow],
		});
		expect(data.repository).toEqual([workflow]);
		expect(data.checks).toEqual([check]);
	});
});
