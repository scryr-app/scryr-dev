import {
	firstBoolean,
	firstNumber,
	firstString,
	getValue,
	type RawRecord,
} from "@/graphql/blockJson";
import type { BlockViewModel } from "@/graphql/blockViewModel";
import type { Block } from "@/graphql/generated";
import type { RuntimeMetricSnapshot } from "../graphql/useDiagramMetrics";
import type { CICDCardProps } from "./CICDCard";
import type { DependenciesCardProps } from "./DependenciesCard";
import type { GithubCardProps } from "./GithubCard";
import type { MetricsCardProps } from "./MetricsCard";
import type { PerformanceCardProps } from "./PerformanceCard";
import { isOperationalReport, type OperationalReport } from "./ReportCard";
import type { TestsCardProps } from "./TestsCard";

function getRepoUrl(
	block: Block,
	raw: RawRecord | undefined,
): string | undefined {
	const githubLink = block.links.find((link) =>
		link.httpUrl?.includes("github.com"),
	)?.httpUrl;

	return (
		block.sourceCodeUrl ??
		githubLink ??
		firstString(raw, [
			["source_code_url"],
			["sourceCodeUrl"],
			["repository", "url"],
			["github", "repoUrl"],
		])
	);
}

function inferBuildStatus(
	explicit?: string,
): "passing" | "failing" | "pending" | undefined {
	if (
		explicit === "passing" ||
		explicit === "failing" ||
		explicit === "pending"
	) {
		return explicit;
	}

	return undefined;
}

export interface BlockCardData {
	runtimeAnalytics?: RuntimeMetricSnapshot;
	runtimeMetrics?: RuntimeMetricSnapshot;
	reports: OperationalReport[];
	github: GithubCardProps;
	metrics: MetricsCardProps;
	cicd: CICDCardProps;
	tests: TestsCardProps;
	dependencies: DependenciesCardProps;
	performance: PerformanceCardProps;
}

export function normalizeBlockCardData(
	block: Block,
	raw: RawRecord | undefined,
): BlockCardData {
	const number = (section: string, key: string) =>
		firstNumber(raw, [[section, key]]);
	const string = (section: string, key: string) =>
		firstString(raw, [[section, key]]);
	const deployStatus = (key: string): CICDCardProps["deployStatusProd"] => {
		const value = string("cicd", key);
		return value === "deployed" || value === "deploying" || value === "failed"
			? value
			: undefined;
	};
	const cpuHistory = getValue(raw, ["performance", "cpuHistory"]);

	return {
		reports: ["tests", "dependencies", "cicd"].flatMap((section) => {
			const value = raw?.[section];
			if (!value || typeof value !== "object" || !("reports" in value))
				return [];
			const reports = value.reports;
			return reports && typeof reports === "object"
				? Object.values(reports).filter(isOperationalReport)
				: [];
		}),
		github: {
			repoUrl: getRepoUrl(block, raw),
			primaryLanguage: block.language ?? undefined,
			stars: firstNumber(raw, [
				["github", "stars"],
				["repository", "stars"],
			]),
			forks: firstNumber(raw, [
				["github", "forks"],
				["repository", "forks"],
			]),
			openIssues: firstNumber(raw, [
				["github", "openIssues"],
				["repository", "openIssues"],
			]),
			openPRs: firstNumber(raw, [
				["github", "openPRs"],
				["repository", "openPRs"],
			]),
			linesOfCode: firstNumber(raw, [
				["github", "linesOfCode"],
				["repository", "linesOfCode"],
			]),
			coverage: number("github", "coverage"),
			vulnerabilities: number("github", "vulnerabilities"),
			outdatedDeps: number("github", "outdatedDeps"),
			activeContributors: number("github", "activeContributors"),
			lastCommit: string("github", "lastCommit"),
			latestRelease: string("github", "latestRelease"),
			license: string("github", "license"),
		},
		metrics: {
			responseTimeP50: firstNumber(raw, [
				["metrics", "responseTimeP50"],
				["metrics", "p50"],
			]),
			responseTimeP95: firstNumber(raw, [
				["metrics", "responseTimeP95"],
				["metrics", "p95"],
			]),
			responseTimeP99: firstNumber(raw, [
				["metrics", "responseTimeP99"],
				["metrics", "p99"],
			]),
			requestRate: number("metrics", "requestRate"),
			errorRate: number("metrics", "errorRate"),
			successRate: number("metrics", "successRate"),
			uptime: number("metrics", "uptime"),
			activeConnections: number("metrics", "activeConnections"),
			cpuUsage: number("metrics", "cpuUsage"),
			memoryUsage: number("metrics", "memoryUsage"),
		},
		cicd: {
			platform: block.cicdTool ?? string("cicd", "platform"),
			buildStatus: inferBuildStatus(
				firstString(raw, [
					["cicd", "buildStatus"],
					["ci", "buildStatus"],
				]),
			),
			lastBuild: firstString(raw, [
				["cicd", "lastBuild"],
				["ci", "lastBuild"],
			]),
			deployStatusProd: deployStatus("deployStatusProd"),
			deployStatusStaging: deployStatus("deployStatusStaging"),
			deployFrequency: firstNumber(raw, [
				["cicd", "deployFrequency"],
				["ci", "deployFrequency"],
			]),
			pipelineDuration: firstNumber(raw, [
				["cicd", "pipelineDuration"],
				["ci", "pipelineDuration"],
			]),
			failedBuilds: firstNumber(raw, [
				["cicd", "failedBuilds"],
				["ci", "failedBuilds"],
			]),
		},
		tests: {
			total: firstNumber(raw, [["tests", "total"]]),
			passing: firstNumber(raw, [["tests", "passing"]]),
			failing: firstNumber(raw, [["tests", "failing"]]),
			coverage: firstNumber(raw, [["tests", "coverage"]]),
			flakyTests: firstNumber(raw, [["tests", "flakyTests"]]),
			executionTime: firstNumber(raw, [["tests", "executionTime"]]),
			lastRun: firstString(raw, [["tests", "lastRun"]]),
			coverageTrend: firstString(raw, [
				["tests", "coverageTrend"],
			]) as TestsCardProps["coverageTrend"],
		},
		dependencies: {
			totalDeps: firstNumber(raw, [["dependencies", "totalDeps"]]),
			outdatedDeps: firstNumber(raw, [["dependencies", "outdatedDeps"]]),
			vulnerableDeps: firstNumber(raw, [["dependencies", "vulnerableDeps"]]),
			directDeps: firstNumber(raw, [["dependencies", "directDeps"]]),
			transitiveDeps: firstNumber(raw, [["dependencies", "transitiveDeps"]]),
			updateLag: firstNumber(raw, [["dependencies", "updateLag"]]),
			maxSeverity: firstString(raw, [
				["dependencies", "maxSeverity"],
			]) as DependenciesCardProps["maxSeverity"],
			licenseCompliance: firstString(raw, [
				["dependencies", "licenseCompliance"],
			]) as DependenciesCardProps["licenseCompliance"],
		},
		performance: {
			cpuHistory: Array.isArray(cpuHistory)
				? cpuHistory.filter(
						(v): v is number => typeof v === "number" && Number.isFinite(v),
					)
				: undefined,
			cpuCurrent: number("performance", "cpuCurrent"),
			cpuAvg: number("performance", "cpuAvg"),
			cpuPeak: number("performance", "cpuPeak"),
			memoryUsage: number("performance", "memoryUsage"),
			timeWindow: string("performance", "timeWindow"),
		},
	};
}

export function hasGraphData(
	block: Block,
	raw: RawRecord | undefined,
): boolean {
	return Boolean(
		block.sourceCodeUrl ||
			block.cicdTool ||
			block.connections.length > 0 ||
			block.frameworks.length > 0 ||
			firstBoolean(raw, [["github", "enabled"]]) ||
			firstBoolean(raw, [["metrics", "enabled"]]),
	);
}

export function getBlockCardData(block: BlockViewModel): BlockCardData {
	return {
		...block.cards,
		runtimeMetrics: block.observations.metrics,
		runtimeAnalytics: block.observations.analytics,
	};
}

export function blockHasGraphData(block: BlockViewModel): boolean {
	return block.hasGraphData;
}
