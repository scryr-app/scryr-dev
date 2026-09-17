import { describe, expect, it, vi } from "vitest";
import { getBlockCardData } from "@/cards/blockCardData";
import { createBlockDataCards } from "./defaultCards";

vi.mock("@/cards", () => ({
	RepositoryCard: () => null,
	ChecksCard: () => null,
	DependenciesCard: () => null,
	MetricsCard: () => null,
	PerformanceCard: () => null,
	TestsCard: () => null,
}));
describe("collector-backed cards", () => {
	it("preserves section indexes while omitting empty sections", () => {
		const empty = getBlockCardData({ rawJsonString: "{}", evidence: [] });
		expect(
			createBlockDataCards(empty).map((group) => group.components.length),
		).toEqual([0, 0, 0, 0, 0, 0]);
		const tests = getBlockCardData({
			rawJsonString: JSON.stringify({
				tests: [{ kind: "pytest", id: "unit" }],
			}),
			evidence: [],
		});
		expect(
			createBlockDataCards(tests).map((group) => group.components.length),
		).toEqual([0, 0, 0, 1, 0, 0]);
	});
	it("uses one card with ordered collector pages rather than dropping additional collectors", () => {
		const data = getBlockCardData({
			rawJsonString: JSON.stringify({
				dependencies: [
					{ kind: "syft_inventory" },
					{ kind: "grant_license" },
					{ kind: "grype_scan" },
				],
			}),
			evidence: [],
		});
		expect(createBlockDataCards(data)[4].components).toHaveLength(1);
		expect(data.dependencies.map((item) => item.integration)).toEqual([
			"syft_inventory",
			"grant_license",
			"grype_scan",
		]);
	});
});
