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

export type BlockCardGroup = Array<{ components: ReactNode[] }>;

export function createDefaultCards(cicdTool?: string): BlockCardGroup {
	return [
		{
			components: [
				<GithubCard
					key="github-card"
					repoUrl="https://github.com/example/repo"
					stars={1247}
					forks={89}
					openIssues={23}
					openPRs={5}
					lastCommit="2 hours ago"
					primaryLanguage="TypeScript"
					linesOfCode={45230}
					coverage={87}
					vulnerabilities={0}
					outdatedDeps={3}
					activeContributors={12}
					latestRelease="v2.4.1"
					license="MIT"
					buildStatus="passing"
				/>,
			],
		},
		{
			components: [
				<MetricsCard
					key="metrics-card"
					responseTimeP50={85}
					responseTimeP95={220}
					responseTimeP99={450}
					requestRate={1250}
					errorRate={0.3}
					successRate={99.7}
					uptime={99.95}
					activeConnections={342}
					cpuUsage={45}
					memoryUsage={62}
				/>,
			],
		},
		{
			components: [
				<CICDCard
					key="cicd-card"
					platform={cicdTool}
					buildStatus="passing"
					lastBuild="12 min ago"
					deployStatusProd="deployed"
					deployStatusStaging="deploying"
					deployFrequency={23}
					pipelineDuration={8}
					failedBuilds={1}
				/>,
			],
		},
		{
			components: [
				<TestsCard
					key="tests-card"
					total={342}
					passing={337}
					failing={2}
					coverage={87}
					coverageTrend="up"
					flakyTests={3}
					executionTime={45}
					lastRun="5 min ago"
				/>,
			],
		},
		{
			components: [
				<DependenciesCard
					key="deps-card"
					totalDeps={342}
					outdatedDeps={12}
					vulnerableDeps={2}
					maxSeverity="medium"
					directDeps={45}
					transitiveDeps={297}
					updateLag={30}
					licenseCompliance="compliant"
				/>,
			],
		},
		{
			components: [
				<PerformanceCard
					key="performance-card"
					cpuHistory={[
						12, 18, 25, 45, 62, 78, 72, 65, 55, 48, 52, 67, 81, 90, 85, 74, 63,
						58, 48, 40,
					]}
					cpuCurrent={40}
					cpuAvg={58.3}
					cpuPeak={90.1}
					memoryUsage={62}
					timeWindow="Last 10 min"
				/>,
			],
		},
	];
}

export function createBlockDataCards(cardData: BlockCardData): BlockCardGroup {
	return [
		{
			components: [<GithubCard key="github-card" {...cardData.github} />],
		},
		{
			components: [<MetricsCard key="metrics-card" {...cardData.metrics} />],
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
				<PerformanceCard key="performance-card" {...cardData.performance} />,
			],
		},
	];
}
