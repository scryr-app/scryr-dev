import { isValidElement } from "react";
import { describe, expect, it, vi } from "vitest";
import type { BlockCardData } from "@/cards/blockCardData";
import { IntegrationCard } from "../cards/IntegrationCard";
import { SetupCard } from "../cards/IntegrationSetup";
import { RuntimeMetricsCard } from "../cards/RuntimeMetricsCard";
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
	it("shows setup cards for empty categories", () => {
		expect(
			createBlockDataCards(empty()).map((group) => group.components.length),
		).toEqual([1, 1, 1, 1, 1, 1, 1, 1]);
	});
	it("shows explicitly reported zeroes only in their category", () => {
		const data = empty();
		data.tests = { failing: 0 };
		const component = createBlockDataCards(data)[3].components[0];
		expect(isValidElement(component) && component.type).not.toBe(SetupCard);
		expect(
			createBlockDataCards(data).map((group) => group.components.length),
		).toEqual([1, 1, 1, 1, 1, 1, 1, 1]);
	});
	it("keeps configured runtime metrics visible", () => {
		const data = empty();
		data.runtimeMetrics = { status: "no_data", values: {} };
		expect(
			createBlockDataCards(data).map((group) => group.components.length),
		).toEqual([1, 1, 1, 1, 1, 1, 1, 1]);
	});
	it("places legacy PostHog observations under Analytics", () => {
		const data = empty();
		data.runtimeAnalytics = { source: "posthog", status: "ready", values: {} };
		const groups = createBlockDataCards(data);
		const analytics = groups.find((group) => group.id === "analytics");
		const component = analytics?.components[0];
		expect(isValidElement(component) && component.type).toBe(
			RuntimeMetricsCard,
		);
		expect(analytics?.categoryIndex).toBe(9);
		const repository = groups.find((group) => group.id === "repository")
			?.components[0];
		expect(isValidElement(repository) && repository.type).toBe(SetupCard);
	});
});

it("renders two provider views independently and respects category order and visibility", () => {
	const data = empty();
	data.cardCategories = ["uptime", "performance"];
	data.cards = [
		{
			id: "grafana_performance",
			kind: "grafana_performance",
			category: "performance",
			title: "Grafana Performance",
			integration: { id: "grafana", kind: "grafana" },
		},
		{
			id: "posthog_performance",
			kind: "posthog_performance",
			category: "performance",
			title: "PostHog Performance",
			integration: { id: "posthog", kind: "posthog" },
		},
	];
	const groups = createBlockDataCards(data).filter((g) => g.components.length);
	expect(groups.map((g) => g.id)).toEqual(["uptime", "performance"]);
	expect(groups[1].categoryIndex).toBe(6);
	expect(
		groups[1].components.map((c) => (isValidElement(c) ? c.key : null)),
	).toEqual(["grafana_performance", "posthog_performance"]);
	expect(
		groups[1].components.every(
			(c) => isValidElement(c) && c.type === IntegrationCard,
		),
	).toBe(true);
});
