import type {
	CollectorEvidenceFieldsFragment,
	EvidenceObservationFieldsFragment,
} from "@/graphql/generated";

export type CollectorEvidence = CollectorEvidenceFieldsFragment;
export type Observation = EvidenceObservationFieldsFragment;
export type EvidenceResult = Observation["result"];
export type EvidenceSection =
	| "repository"
	| "checks"
	| "metrics"
	| "tests"
	| "dependencies"
	| "performance";
export const EVIDENCE_SECTIONS: EvidenceSection[] = [
	"repository",
	"checks",
	"metrics",
	"tests",
	"dependencies",
	"performance",
];
export const SECTION_TITLES: Record<EvidenceSection, string> = {
	repository: "Repository",
	checks: "Checks",
	metrics: "Metrics",
	tests: "Tests",
	dependencies: "Dependencies",
	performance: "Performance",
};
const INTEGRATIONS: Record<string, string> = {
	git_status: "Git",
	github_pull_requests: "GitHub pull requests",
	github_actions: "GitHub Actions",
	ruff: "Ruff",
	biome: "Biome",
	clippy: "Cargo Clippy",
	mise_task: "mise task",
	openmetrics: "OpenMetrics",
	docker_stats: "Docker stats",
	pytest: "pytest",
	vitest: "Vitest",
	nextest: "Nextest",
	junit: "JUnit",
	lcov: "LCOV",
	cobertura: "Cobertura",
	syft_inventory: "Syft inventory",
	grype_scan: "Grype vulnerabilities",
	grant_license: "Grant licenses",
	hyperfine: "Hyperfine",
};
export function integrationTitle(integration: string): string {
	return INTEGRATIONS[integration] ?? integration;
}
export function sectionName(
	section: CollectorEvidence["section"],
): EvidenceSection {
	return section.toLowerCase() as EvidenceSection;
}
export function collectorKey(collector: CollectorEvidence): string {
	return `${collector.manifestId}:${collector.section}:${collector.collectorId}:${collector.workspaceId}`;
}

export function collectorStatus(collector: CollectorEvidence): string {
	const states: Record<CollectorEvidence["state"], string> = {
		WAITING: "Waiting for first result",
		RUNNING: "Running",
		READY: "Collected",
		MISSING_TOOL: "Missing tool",
		INCOMPATIBLE_TOOL: "Unsupported tool version",
		NEEDS_LOGIN: "Authentication needed",
		ERROR: "Collection failed",
		CANCELLED: "Cancelled",
		DISABLED: "Paused",
	};
	return [
		states[collector.state],
		collector.outdated ? "Outdated inputs" : collector.stale ? "Stale" : null,
	]
		.filter(Boolean)
		.join(" · ");
}
export function statusTone(
	collector: CollectorEvidence,
): "neutral" | "warning" | "error" {
	if (
		["ERROR", "MISSING_TOOL", "INCOMPATIBLE_TOOL", "NEEDS_LOGIN"].includes(
			collector.state,
		)
	)
		return "error";
	if (collector.stale || collector.outdated) return "warning";
	return "neutral";
}
export function formatNumber(value: number): string {
	return Number.isFinite(value)
		? new Intl.NumberFormat("en", { maximumFractionDigits: 3 }).format(value)
		: "Unavailable";
}
export function formatSeconds(value: number): string {
	return value < 1
		? `${formatNumber(value * 1000)} ms`
		: `${formatNumber(value)} s`;
}
export function metricLabel(
	sample: Extract<
		EvidenceResult,
		{ __typename: "MetricsResult" }
	>["samples"][number],
): string {
	const labels = sample.labels
		.map((label) => `${label.name}=${label.value}`)
		.join(", ");
	return `${sample.title ?? sample.name}${labels ? ` {${labels}}` : ""}`;
}
export function detectedLicenses(licenses: string[]): string[] {
	return licenses.filter(
		(license) =>
			!["", "UNKNOWN", "NOASSERTION", "NONE"].includes(
				license.trim().toUpperCase(),
			),
	);
}
export function resultSummary(result: EvidenceResult): string[] {
	switch (result.__typename) {
		case "GitResult":
			return [
				`${result.branch ?? "Detached HEAD"} · ${result.commit?.slice(0, 8) ?? "No commit"}`,
				result.dirty
					? `${result.changedFiles} changed files`
					: "Working tree clean",
				`${result.ahead} ahead · ${result.behind} behind`,
			];
		case "PullRequestsResult":
			return [
				`${result.items.length} pull requests${result.complete ? "" : " · Partial listing"}`,
				...result.items
					.slice(0, 2)
					.map(
						(pr) =>
							`#${pr.number} ${pr.title} · ${pr.reviewDecision ?? pr.state}`,
					),
			];
		case "WorkflowsResult":
			return [
				`Remote GitHub workflows${result.complete ? "" : " · Partial listing"}`,
				...result.items
					.slice(0, 2)
					.map(
						(run) =>
							`${run.name}: ${run.conclusion ?? run.status} · ${run.branch}`,
					),
				...(!result.items.length ? ["No workflow runs returned"] : []),
			];
		case "CheckResult":
			return [
				`${result.name}: ${result.passed ? "Passed" : "Failed"}`,
				`${result.diagnostics.length} diagnostics · ${formatSeconds(result.durationSeconds)}`,
				...result.diagnostics.slice(0, 1).map((d) => d.message),
			];
		case "TestResult":
			return [
				`${result.suite}: ${result.passing} passed · ${result.failing} failed`,
				`${result.errors} errors · ${result.skipped} skipped`,
				formatSeconds(result.durationSeconds),
			];
		case "CoverageResult":
			return [
				`${result.suite} coverage`,
				result.total > 0
					? `${formatNumber((100 * result.covered) / result.total)}% · ${result.covered}/${result.total} lines`
					: "Coverage unavailable: no measured lines",
			];
		case "InventoryResult":
			return [
				`${result.totalPackages} packages${result.complete ? "" : " · Incomplete inventory"}`,
				`${result.unknownLicensePackages} packages with unknown licenses`,
				"License evidence · Policy not evaluated",
			];
		case "LicenseResult":
			return [
				`${result.deniedCount} denied · ${result.reviewCount} need review`,
				`${result.allowedCount} allowed`,
				result.complete
					? "License policy evaluated"
					: "Incomplete license evaluation",
			];
		case "VulnerabilityResult":
			return [
				`${result.totalFindings} advisory/package findings`,
				`${result.criticalCount + result.highCount} high or critical`,
				result.complete
					? result.totalFindings
						? "Review fixes in details"
						: "No findings in completed scan"
					: "Incomplete scan · Security status unknown",
			];
		case "MetricsResult":
			return [
				...(!result.complete
					? ["Partial scrape · Some series unavailable"]
					: []),
				...result.samples
					.slice(0, result.complete ? 3 : 2)
					.map(
						(sample) =>
							`${metricLabel(sample)}: ${formatNumber(sample.value)} ${sample.unit ?? ""}`,
					),
				...(!result.samples.length ? ["No computed samples yet"] : []),
			];
		case "BenchmarkResult":
			return [
				`${result.name}: ${formatSeconds(result.meanSeconds)} mean`,
				`${formatSeconds(result.medianSeconds)} median · ${result.runs} runs`,
				result.baselineMeanSeconds && result.baselineMeanSeconds > 0
					? `${formatNumber((result.meanSeconds / result.baselineMeanSeconds - 1) * 100)}% vs baseline`
					: "No comparable baseline",
			];
	}
}
export function safeHttpUrl(value?: string | null): string | undefined {
	if (!value) return undefined;
	try {
		const url = new URL(value);
		return ["https:", "http:"].includes(url.protocol) ? url.href : undefined;
	} catch {
		return undefined;
	}
}
export function collectCommand(collector: CollectorEvidence): string {
	const quote = (value: string) => `'${value.replaceAll("'", "'\\''")}'`;
	return `scryr collect run --manifest ${quote(collector.manifestId)} --section ${sectionName(collector.section)} --collector ${quote(collector.collectorId)}`;
}
