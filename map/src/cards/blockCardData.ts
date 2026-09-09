import type { Block } from "@/graphql/generated";
import type { CICDCardProps } from "./CICDCard";
import type { DependenciesCardProps } from "./DependenciesCard";
import type { GithubCardProps } from "./GithubCard";
import type { MetricsCardProps } from "./MetricsCard";
import type { PerformanceCardProps } from "./PerformanceCard";
import { isOperationalReport, type OperationalReport } from "./ReportCard";
import type { TestsCardProps } from "./TestsCard";

type RawRecord = Record<string, unknown>;

function clamp(value: number, min: number, max: number): number {
	return Math.min(max, Math.max(min, value));
}

function hashString(value: string): number {
	let hash = 2166136261;

	for (let index = 0; index < value.length; index += 1) {
		hash ^= value.charCodeAt(index);
		hash = Math.imul(hash, 16777619);
	}

	return Math.abs(hash >>> 0);
}

function seededInt(
	seed: string,
	slot: string,
	min: number,
	max: number,
): number {
	const hash = hashString(`${seed}:${slot}`);
	return min + (hash % (max - min + 1));
}

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

function firstStringArray(
	source: RawRecord | undefined,
	paths: string[][],
): string[] | undefined {
	for (const path of paths) {
		const value = getValue(source, path);
		if (Array.isArray(value)) {
			return value.filter((item): item is string => typeof item === "string");
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

function inferDeployStatus(isProd = true): "deployed" | "deploying" {
	return isProd ? "deployed" : "deploying";
}

function buildCpuHistory(seed: string, current: number): number[] {
	return Array.from({ length: 20 }, (_, index) => {
		const drift = seededInt(seed, `cpu-history-${index}`, -12, 14);
		return clamp(current + drift, 10, 95);
	});
}

export interface BlockCardData {
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
	const seed = block.rawJsonString || block.name || "block";
	const tagCount = block.tags.length;
	const frameworkCount = block.frameworks.length;
	const connectionCount = block.connections.length;
	const docsCount = block.docs.length;
	const linkCount = block.links.length;
	const replicaCount = Math.max(
		block.maxReplicas ?? 1,
		block.minReplicas ?? 1,
		1,
	);
	const complexity =
		tagCount + frameworkCount * 2 + connectionCount * 3 + docsCount + linkCount;
	const repoUrl = getRepoUrl(block, raw);
	const buildStatus = inferBuildStatus(
		firstString(raw, [
			["ci", "buildStatus"],
			["cicd", "buildStatus"],
		]),
	);
	const coverage =
		firstNumber(raw, [
			["quality", "coverage"],
			["tests", "coverage"],
			["github", "coverage"],
		]) ?? clamp(74 + frameworkCount * 3 + connectionCount, 52, 98);
	const cpuCurrent =
		firstNumber(raw, [
			["performance", "cpuCurrent"],
			["metrics", "cpuUsage"],
		]) ??
		clamp(
			22 +
				complexity * 3 +
				replicaCount * 4 +
				seededInt(seed, "cpu-current", -10, 16),
			12,
			92,
		);
	const memoryUsage =
		firstNumber(raw, [
			["performance", "memoryUsage"],
			["metrics", "memoryUsage"],
		]) ?? clamp(cpuCurrent + seededInt(seed, "memory-usage", -8, 20), 18, 94);
	const errorRate =
		firstNumber(raw, [["metrics", "errorRate"]]) ??
		Number((0.4 + seededInt(seed, "error-rate", 0, 8) / 10).toFixed(1));
	const uptime =
		firstNumber(raw, [["metrics", "uptime"]]) ??
		Number(
			clamp(
				99.05 + seededInt(seed, "uptime", 0, 18) / 100,
				97.4,
				99.99,
			).toFixed(2),
		);
	const lastBuildHours = seededInt(seed, "last-build-hours", 1, 72);
	const lastBuild =
		firstString(raw, [
			["ci", "lastBuild"],
			["cicd", "lastBuild"],
		]) ??
		(lastBuildHours < 24
			? `${lastBuildHours}h ago`
			: `${Math.floor(lastBuildHours / 24)}d ago`);
	const deployFrequency =
		firstNumber(raw, [
			["ci", "deployFrequency"],
			["cicd", "deployFrequency"],
		]) ??
		(block.cicdTool === "none"
			? 0
			: clamp(
					1 +
						frameworkCount +
						replicaCount +
						seededInt(seed, "deploy-frequency", 0, 6),
					1,
					20,
				));
	const pipelineDuration =
		firstNumber(raw, [
			["ci", "pipelineDuration"],
			["cicd", "pipelineDuration"],
		]) ??
		clamp(
			6 + complexity * 2 + seededInt(seed, "pipeline-duration", 0, 18),
			4,
			42,
		);
	const vulnerableDeps = firstNumber(raw, [["dependencies", "vulnerableDeps"]]);
	const outdatedDeps = firstNumber(raw, [["dependencies", "outdatedDeps"]]);
	const cpuHistory =
		firstStringArray(raw, [["performance", "cpuHistory"]])
			?.map((value) => Number(value))
			.filter((value) => Number.isFinite(value)) ??
		buildCpuHistory(seed, cpuCurrent);

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
			repoUrl,
			stars:
				firstNumber(raw, [
					["github", "stars"],
					["repository", "stars"],
				]) ??
				clamp(
					100 + complexity * 25 + seededInt(seed, "stars", 0, 1800),
					40,
					9000,
				),
			forks:
				firstNumber(raw, [
					["github", "forks"],
					["repository", "forks"],
				]) ??
				clamp(12 + complexity * 4 + seededInt(seed, "forks", 0, 180), 4, 1200),
			openIssues:
				firstNumber(raw, [
					["github", "openIssues"],
					["repository", "openIssues"],
				]) ??
				clamp(connectionCount + seededInt(seed, "open-issues", 0, 18), 0, 48),
			openPRs:
				firstNumber(raw, [
					["github", "openPRs"],
					["repository", "openPRs"],
				]) ?? clamp(frameworkCount + seededInt(seed, "open-prs", 0, 8), 0, 16),
			lastCommit:
				firstString(raw, [
					["github", "lastCommit"],
					["repository", "lastCommit"],
				]) ?? lastBuild,
			primaryLanguage: block.language ?? undefined,
			linesOfCode:
				firstNumber(raw, [
					["github", "linesOfCode"],
					["repository", "linesOfCode"],
				]) ??
				clamp(
					4000 + complexity * 1600 + seededInt(seed, "loc", 0, 12000),
					3000,
					180000,
				),
			coverage,
			vulnerabilities:
				firstNumber(raw, [
					["github", "vulnerabilities"],
					["security", "vulnerabilities"],
				]) ?? vulnerableDeps,
			outdatedDeps,
			activeContributors:
				firstNumber(raw, [["github", "activeContributors"]]) ??
				clamp(
					2 +
						frameworkCount +
						connectionCount +
						seededInt(seed, "contributors", 0, 6),
					1,
					18,
				),
			latestRelease: block.version ?? undefined,
			license: firstString(raw, [["license"], ["github", "license"]]),
			buildStatus,
		},
		metrics: {
			responseTimeP50:
				firstNumber(raw, [
					["metrics", "responseTimeP50"],
					["metrics", "p50"],
				]) ??
				clamp(40 + complexity * 5 + seededInt(seed, "p50", 0, 40), 25, 260),
			responseTimeP95:
				firstNumber(raw, [
					["metrics", "responseTimeP95"],
					["metrics", "p95"],
				]) ??
				clamp(120 + complexity * 8 + seededInt(seed, "p95", 0, 90), 80, 520),
			responseTimeP99:
				firstNumber(raw, [
					["metrics", "responseTimeP99"],
					["metrics", "p99"],
				]) ??
				clamp(
					220 + complexity * 12 + seededInt(seed, "p99", 0, 180),
					150,
					1200,
				),
			requestRate:
				firstNumber(raw, [["metrics", "requestRate"]]) ??
				clamp(
					120 +
						connectionCount * 140 +
						replicaCount * 180 +
						seededInt(seed, "request-rate", 0, 480),
					60,
					4200,
				),
			errorRate,
			successRate: Number(clamp(100 - errorRate, 91, 99.9).toFixed(1)),
			uptime,
			activeConnections:
				firstNumber(raw, [["metrics", "activeConnections"]]) ??
				clamp(
					connectionCount * 40 +
						replicaCount * 70 +
						seededInt(seed, "connections", 0, 140),
					20,
					1200,
				),
			cpuUsage: firstNumber(raw, [["metrics", "cpuUsage"]]) ?? cpuCurrent,
			memoryUsage,
		},
		cicd: {
			platform: block.cicdTool ?? undefined,
			buildStatus,
			lastBuild,
			deployStatusProd:
				(firstString(raw, [["cicd", "deployStatusProd"]]) as
					| "deployed"
					| "deploying"
					| "failed"
					| undefined) ?? inferDeployStatus(true),
			deployStatusStaging:
				(firstString(raw, [["cicd", "deployStatusStaging"]]) as
					| "deployed"
					| "deploying"
					| "failed"
					| undefined) ?? inferDeployStatus(false),
			deployFrequency,
			pipelineDuration,
			failedBuilds:
				firstNumber(raw, [
					["cicd", "failedBuilds"],
					["ci", "failedBuilds"],
				]) ??
				(buildStatus === "failing"
					? seededInt(seed, "failed-builds", 1, 4)
					: 0),
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
			cpuHistory,
			cpuCurrent,
			cpuAvg:
				firstNumber(raw, [["performance", "cpuAvg"]]) ??
				Number(
					(
						cpuHistory.reduce((sum, value) => sum + value, 0) /
						cpuHistory.length
					).toFixed(1),
				),
			cpuPeak:
				firstNumber(raw, [["performance", "cpuPeak"]]) ??
				Math.max(...cpuHistory),
			memoryUsage,
			timeWindow:
				firstString(raw, [["performance", "timeWindow"]]) ?? "Last 10 min",
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
