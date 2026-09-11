import type { Block } from "@/graphql/generated";
import type { RuntimeMetricSnapshot } from "../graphql/useDiagramMetrics";
import type { CICDCardProps } from "./CICDCard";
import type { DependenciesCardProps } from "./DependenciesCard";
import type { GithubCardProps } from "./GithubCard";
import {
	type ActionRun,
	type CardCategory,
	type IntegrationView,
	isIntegrationView,
} from "./integrationCatalog";
import type { MetricsCardProps } from "./MetricsCard";
import type { PerformanceCardProps } from "./PerformanceCard";
import { isOperationalReport, type OperationalReport } from "./ReportCard";
import type { TestsCardProps } from "./TestsCard";

type RawRecord = Record<string, unknown>;

function asRecord(value: unknown): RawRecord | undefined {
	return value && typeof value === "object" && !Array.isArray(value)
		? (value as RawRecord)
		: undefined;
}

function parseRawJsonString(
	rawJsonString?: string | null,
): RawRecord | undefined {
	if (!rawJsonString) {
		return undefined;
	}

	try {
		return asRecord(JSON.parse(rawJsonString));
	} catch {
		return undefined;
	}
}

function getValue(source: RawRecord | undefined, path: string[]): unknown {
	let current: unknown = source;

	for (const part of path) {
		const record = asRecord(current);
		if (!record) {
			return undefined;
		}
		current = record[part];
	}

	return current;
}

function firstString(
	source: RawRecord | undefined,
	paths: string[][],
): string | undefined {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "string" && value.length > 0) {
			return value;
		}
	}

	return undefined;
}

function firstNumber(
	source: RawRecord | undefined,
	paths: string[][],
): number | undefined {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "number" && Number.isFinite(value)) {
			return value;
		}
	}

	return undefined;
}

function firstBoolean(
	source: RawRecord | undefined,
	paths: string[][],
): boolean | undefined {
	for (const path of paths) {
		const value = getValue(source, path);
		if (typeof value === "boolean") {
			return value;
		}
	}

	return undefined;
}

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
	cards?: IntegrationView[];
	cardCategories?: CardCategory[];
	runtimeCards?: Record<string, RuntimeMetricSnapshot>;
	actionRuns?: ActionRun[];
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

export function getBlockCardData(block: Block): BlockCardData {
	const raw = parseRawJsonString(block.rawJsonString);
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
		cards: Array.isArray(raw?.cards)
			? raw.cards.filter(isIntegrationView)
			: undefined,
		cardCategories: Array.isArray(raw?.cardCategories)
			? (raw.cardCategories as CardCategory[])
			: undefined,
		runtimeCards: asRecord(raw?.runtimeCards) as
			| Record<string, RuntimeMetricSnapshot>
			| undefined,
		actionRuns:
			(getValue(raw, ["cicd", "githubActions", "runs"]) as
				| ActionRun[]
				| undefined) ?? [],
		runtimeAnalytics: raw?.runtimeAnalytics as
			| RuntimeMetricSnapshot
			| undefined,
		runtimeMetrics: raw?.runtimeMetrics as RuntimeMetricSnapshot | undefined,
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

export function blockHasGraphData(block: Block): boolean {
	const raw = parseRawJsonString(block.rawJsonString);

	return Boolean(
		block.sourceCodeUrl ||
			block.cicdTool ||
			block.connections.length > 0 ||
			block.frameworks.length > 0 ||
			firstBoolean(raw, [["github", "enabled"]]) ||
			firstBoolean(raw, [["metrics", "enabled"]]),
	);
}
