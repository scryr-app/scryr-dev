import type { ReactNode } from "react";
import {
	CICDCard,
	DependenciesCard,
	GithubCard,
	MetricsCard,
	PerformanceCard,
	TestsCard,
} from "@/cards";
import type { BlockCardData } from "@/cards/blockCardData";
import { IntegrationCard } from "../cards/IntegrationCard";
import { SetupCard } from "../cards/IntegrationSetup";
import { integrationCategories } from "../cards/integrationCatalog";
import { ReportCard } from "../cards/ReportCard";
import { RuntimeMetricsCard } from "../cards/RuntimeMetricsCard";

export type BlockCardGroup = Array<{
	categoryIndex?: number;
	id?: string;
	label?: string;
	components: ReactNode[];
}>;

function hasData(props: object): boolean {
	return Object.values(props).some((value) =>
		Array.isArray(value)
			? value.length > 0
			: value !== undefined && value !== null && value !== "",
	);
}

function reportKey(
	report: BlockCardData["reports"][number],
	index: number,
): string {
	return [
		report.data.kind,
		report.scope,
		report.source,
		report.runId,
		report.attempt,
		index,
	].join(":");
}

export function createBlockDataCards(cardData: BlockCardData): BlockCardGroup {
	const legacy: BlockCardGroup = [
		{
			components: [
				...(hasData(cardData.github)
					? [<GithubCard key="github-card" {...cardData.github} />]
					: []),
			],
		},
		{
			components: cardData.runtimeMetrics
				? [
						<RuntimeMetricsCard
							key="metrics-card"
							snapshot={cardData.runtimeMetrics}
						/>,
					]
				: hasData(cardData.metrics)
					? [<MetricsCard key="metrics-card" {...cardData.metrics} />]
					: [],
		},
		{
			components: [
				...(hasData(cardData.cicd)
					? [<CICDCard key="cicd-card" {...cardData.cicd} />]
					: []),
				...cardData.reports
					.filter((r) => r.data.kind === "deployment")
					.map((r, index) => (
						<ReportCard key={reportKey(r, index)} report={r} />
					)),
			],
		},
		{
			components: cardData.reports.some(
				(r) => r.data.kind === "tests" || r.data.kind === "coverage",
			)
				? cardData.reports
						.filter(
							(r) => r.data.kind === "tests" || r.data.kind === "coverage",
						)
						.map((r, index) => (
							<ReportCard key={reportKey(r, index)} report={r} />
						))
				: hasData(cardData.tests)
					? [<TestsCard key="tests-card" {...cardData.tests} />]
					: [],
		},
		{
			components: cardData.reports.some((r) => r.data.kind === "dependencies")
				? cardData.reports
						.filter((r) => r.data.kind === "dependencies")
						.map((r, index) => (
							<ReportCard key={reportKey(r, index)} report={r} />
						))
				: hasData(cardData.dependencies)
					? [<DependenciesCard key="deps-card" {...cardData.dependencies} />]
					: [],
		},
		{
			components:
				cardData.runtimeMetrics || hasData(cardData.performance)
					? [
							cardData.runtimeMetrics ? (
								<RuntimeMetricsCard
									key="performance-card"
									snapshot={cardData.runtimeMetrics}
									performance
								/>
							) : (
								<PerformanceCard
									key="performance-card"
									{...cardData.performance}
								/>
							),
						]
					: [],
		},
		{ components: [] },
		{
			components: cardData.runtimeAnalytics
				? [
						<RuntimeMetricsCard
							key="analytics-card"
							snapshot={cardData.runtimeAnalytics}
						/>,
					]
				: [],
		},
	];
	return integrationCategories
		.map((category, index) => {
			const declared = cardData.cards?.filter(
				(card) => card.category === category.id,
			);
			const components = declared?.length
				? declared.map((card) => (
						<IntegrationCard
							key={card.id}
							card={card}
							snapshot={cardData.runtimeCards?.[card.id]}
							runs={cardData.actionRuns}
						/>
					))
				: cardData.cards !== undefined &&
						!["repository", "tests", "dependencies"].includes(category.id)
					? []
					: (legacy[index]?.components ?? []);
			const visible =
				!cardData.cardCategories ||
				cardData.cardCategories.includes(category.id);
			return {
				categoryIndex: index < 6 ? index + 1 : index + 2,
				id: category.id,
				label: category.label,
				components: !visible
					? []
					: components.length
						? components
						: [
								<SetupCard
									key={`setup-${category.id}`}
									category={category.id}
								/>,
							],
			};
		})
		.sort((a, b) => {
			const order =
				cardData.cardCategories ?? integrationCategories.map((c) => c.id);
			return order.indexOf(a.id) - order.indexOf(b.id);
		});
}
