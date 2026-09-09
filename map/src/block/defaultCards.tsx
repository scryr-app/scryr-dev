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

export function createDefaultCards(cicdTool?: string): BlockCardGroup {
	return [
		{ components: [<GithubCard key="github-card" />] },
		{ components: [<MetricsCard key="metrics-card" />] },
		{ components: [<CICDCard key="cicd-card" platform={cicdTool} />] },
		{ components: [<TestsCard key="tests-card" />] },
		{ components: [<DependenciesCard key="deps-card" />] },
		{ components: [<PerformanceCard key="performance-card" />] },
	];
}

export function createBlockDataCards(cardData: BlockCardData): BlockCardGroup {
	return [
		{
			components: [
				<GithubCard key="github-card" {...cardData.github} />,
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
				: [<MetricsCard key="metrics-card" {...cardData.metrics} />],
		},
		{
			components: [
				<CICDCard key="cicd-card" {...cardData.cicd} />,
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
				: [<TestsCard key="tests-card" {...cardData.tests} />],
		},
		{
			components: cardData.reports.some((r) => r.data.kind === "dependencies")
				? cardData.reports
						.filter((r) => r.data.kind === "dependencies")
						.map((r) => <ReportCard key={r.scope} report={r} />)
				: [<DependenciesCard key="deps-card" {...cardData.dependencies} />],
		},
		{
			components: [
				cardData.runtimeMetrics ? (
					<RuntimeMetricsCard
						key="performance-card"
						snapshot={cardData.runtimeMetrics}
						performance
					/>
				) : (
					<PerformanceCard key="performance-card" {...cardData.performance} />
				),
			],
		},
	];
}
