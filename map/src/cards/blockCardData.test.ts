import { describe, expect, it } from "vitest";
import type { Block } from "@/graphql/generated";
import { getBlockCardData } from "./blockCardData";

function block(raw: unknown = {}): Block {
	return {
		name: "api",
		links: [],
		tags: ["api"],
		frameworks: [],
		connections: [],
		docs: [],
		version: "1.0.0",
		rawJsonString: JSON.stringify(raw),
	} as unknown as Block;
}

describe("reported card data", () => {
	it("does not invent observations from architecture metadata", () => {
		const data = getBlockCardData(block());
		for (const section of [
			data.github,
			data.metrics,
			data.cicd,
			data.tests,
			data.dependencies,
			data.performance,
		]) {
			expect(Object.values(section).every((value) => value === undefined)).toBe(
				true,
			);
		}
		expect(data.reports).toEqual([]);
	});
	it("preserves real zeroes and numeric history without estimating missing fields", () => {
		const data = getBlockCardData(
			block({
				github: { stars: 0 },
				metrics: { errorRate: 0 },
				performance: { cpuHistory: [0, 23, "45", null] },
				cicd: { failedBuilds: 0, deployStatusProd: "bogus" },
			}),
		);
		expect(data.github.stars).toBe(0);
		expect(data.metrics.errorRate).toBe(0);
		expect(data.metrics.successRate).toBeUndefined();
		expect(data.performance.cpuHistory).toEqual([0, 23]);
		expect(data.performance.cpuAvg).toBeUndefined();
		expect(data.cicd.failedBuilds).toBe(0);
		expect(data.cicd.deployStatusProd).toBeUndefined();
	});
});
