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
import { ReportCard } from "../cards/ReportCard";
import { RuntimeMetricsCard } from "../cards/RuntimeMetricsCard";

export type BlockCardGroup = Array<{ components: ReactNode[] }>;

function hasData(props: object): boolean {
	return Object.values(props).some((value) =>
		Array.isArray(value)
			? value.length > 0
			: value !== undefined && value !== null && value !== "",
	);
}

export function createBlockDataCards(cardData: BlockCardData): BlockCardGroup {
	return [
		{
			components: [
				...(hasData(cardData.github)
					? [<GithubCard key="github-card" {...cardData.github} />]
					: []),
				...(cardData.runtimeAnalytics
					? [
							<RuntimeMetricsCard
								key="analytics-card"
								snapshot={cardData.runtimeAnalytics}
							/>,
						]
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
					.map((r) => <ReportCard key={r.scope} report={r} />),
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
						.map((r) => (
							<ReportCard key={`${r.data.kind}:${r.scope}`} report={r} />
						))
				: hasData(cardData.tests)
					? [<TestsCard key="tests-card" {...cardData.tests} />]
					: [],
		},
		{
			components: cardData.reports.some((r) => r.data.kind === "dependencies")
				? cardData.reports
						.filter((r) => r.data.kind === "dependencies")
						.map((r) => <ReportCard key={r.scope} report={r} />)
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
	];
}
