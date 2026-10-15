import { describe, expect, it, vi } from "vitest";
import type { BlockCardData } from "@/cards/blockCardData";
import { createBlockDataCards } from "./defaultCards";

vi.mock("@/cards", () => ({
	CICDCard: () => null,
	DependenciesCard: () => null,
	GithubCard: () => null,
	MetricsCard: () => null,
	PerformanceCard: () => null,
	TestsCard: () => null,
}));
vi.mock("../cards/ReportCard", () => ({ ReportCard: () => null }));
vi.mock("../cards/RuntimeMetricsCard", () => ({
	RuntimeMetricsCard: () => null,
}));

const empty = (): BlockCardData => ({
	reports: [],
	github: {},
	metrics: {},
	cicd: {},
	tests: {},
	dependencies: {},
	performance: { cpuHistory: [] },
});

describe("manifest-backed cards", () => {
	it("keeps absent categories empty without changing tray indexes", () => {
		expect(
			createBlockDataCards(empty()).map((group) => group.components.length),
		).toEqual([0, 0, 0, 0, 0, 0]);
	});
	it("shows explicitly reported zeroes only in their category", () => {
		const data = empty();
		data.tests = { failing: 0 };
		expect(
			createBlockDataCards(data).map((group) => group.components.length),
		).toEqual([0, 0, 0, 1, 0, 0]);
	});
	it("keeps configured runtime metrics visible", () => {
		const data = empty();
		data.runtimeMetrics = { status: "no_data", values: {} };
		expect(
			createBlockDataCards(data).map((group) => group.components.length),
		).toEqual([0, 1, 0, 0, 0, 1]);
	});
});
