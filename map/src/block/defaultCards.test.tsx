import { isValidElement } from "react";
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
vi.mock("../cards/GithubActionsCard", () => ({
	GithubActionsCard: () => null,
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

describe("GitHub Actions cards", () => {
	it("passes deployment and pipeline data to the visible provider card", () => {
		const data = empty();
		data.cicd = { buildStatus: "passing", deployStatusProd: "deployed" };
		data.githubActions = {
			workflows: [],
			sync: { error: "gh is unavailable" },
		};
		const groups = createBlockDataCards(data);
		expect(groups.map((group) => group.components.length)).toEqual([
			0, 0, 1, 0, 0, 0,
		]);
		expect(
			groups[2].components.map((component) =>
				isValidElement(component) ? component.key : null,
			),
		).toEqual(["github-actions-card"]);
		const card = groups[2].components[0];
		expect(
			isValidElement<{ pipeline: unknown }>(card) && card.props.pipeline,
		).toEqual(data.cicd);
	});
	it("shows an initial collection failure even before any builds exist", () => {
		const data = empty();
		data.githubActions = {
			workflows: [],
			sync: { error: "gh is unavailable" },
		};
		expect(createBlockDataCards(data)[2].components).toHaveLength(1);
	});
});
