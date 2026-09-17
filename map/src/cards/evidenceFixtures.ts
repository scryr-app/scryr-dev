import type {
	CollectorEvidence,
	EvidenceResult,
	Observation,
} from "./evidence";

/** Representative fixtures shared by card and details regression tests. */
export const results = {
	git: {
		__typename: "GitResult",
		branch: "feature/cards",
		commit: "abc123456789",
		dirty: true,
		changedFiles: 2,
		ahead: 1,
		behind: 0,
		remoteUrl: "https://github.com/example/app",
	},
	pullRequests: {
		__typename: "PullRequestsResult",
		repository: "example/app",
		complete: true,
		items: [
			{
				number: 42,
				title: "CLI collector cards",
				url: "https://github.com/example/app/pull/42",
				state: "OPEN",
				reviewDecision: "REVIEW_REQUIRED",
				headRefName: "feature/cards",
				headRefOid: "abc123456789",
			},
		],
	},
	workflows: {
		__typename: "WorkflowsResult",
		repository: "example/app",
		complete: true,
		items: [
			{
				runId: "123",
				attempt: 2,
				name: "Verify",
				status: "completed",
				conclusion: "failure",
				branch: "main",
				commit: "remote4567",
				url: "https://github.com/example/app/actions/runs/123",
				updatedAt: "2026-09-17T10:00:00Z",
			},
		],
	},
	check: {
		__typename: "CheckResult",
		name: "Ruff",
		passed: false,
		durationSeconds: 0.8,
		diagnostics: [
			{
				message: "Unused import",
				path: "src/app.py",
				line: 3,
				severity: "error",
			},
		],
	},
	tests: {
		__typename: "TestResult",
		suite: "unit",
		passing: 48,
		failing: 1,
		errors: 0,
		skipped: 2,
		durationSeconds: 3.4,
		cases: [
			{
				name: "can save",
				suite: "unit",
				status: "failed",
				message: "Expected persisted result",
				durationSeconds: 0.2,
			},
		],
	},
	coverage: {
		__typename: "CoverageResult",
		suite: "unit",
		covered: 80,
		total: 100,
	},
	inventory: {
		__typename: "InventoryResult",
		totalPackages: 2,
		totalRelationships: 1,
		licensedPackages: 1,
		unknownLicensePackages: 1,
		complete: true,
		artifactHash: "inventory-1",
		packages: [
			{
				id: "pkg-a",
				name: "sample-lib",
				version: "1.0",
				ecosystem: "npm",
				purl: "pkg:npm/sample-lib@1.0",
				paths: ["package-lock.json"],
				licenses: ["MIT"],
			},
			{
				id: "pkg-b",
				name: "unknown-lib",
				version: "2.0",
				ecosystem: "pypi",
				purl: null,
				paths: ["uv.lock"],
				licenses: [],
			},
		],
		relationships: [{ from: "pkg-a", to: "pkg-b" }],
	},
	licenses: {
		__typename: "LicenseResult",
		totalItems: 2,
		allowedCount: 1,
		deniedCount: 0,
		reviewCount: 1,
		complete: true,
		policyRevision: "policy-1",
		inventoryHash: "inventory-1",
		items: [
			{
				packageId: "pkg-a",
				expression: "MIT",
				decision: "allow",
				reason: "Allowed SPDX expression",
			},
			{
				packageId: "pkg-b",
				expression: null,
				decision: "review",
				reason: "Missing license evidence",
			},
		],
	},
	vulnerabilities: {
		__typename: "VulnerabilityResult",
		totalFindings: 1,
		affectedPackages: 1,
		criticalCount: 0,
		highCount: 1,
		mediumCount: 0,
		lowCount: 0,
		unknownSeverityCount: 0,
		complete: true,
		inventoryHash: "inventory-1",
		databaseAgeSeconds: 600,
		databaseVersion: "2026-09-17",
		items: [
			{
				packageId: "pkg-a",
				advisoryId: "CVE-2026-1234",
				aliases: ["GHSA-abcd"],
				severity: "high",
				fixVersions: ["1.1"],
				advisoryUrl: "https://example.com/CVE-2026-1234",
			},
		],
	},
	metrics: {
		__typename: "MetricsResult",
		complete: true,
		scrapedAt: "2026-09-17T10:00:00Z",
		samples: [
			{
				name: "http_requests_total",
				title: "Request rate",
				labels: [{ name: "route", value: "/api" }],
				value: 0,
				unit: "requests/s",
				metricType: "counter_rate",
			},
		],
	},
	benchmark: {
		__typename: "BenchmarkResult",
		name: "Startup",
		meanSeconds: 0.15,
		stddevSeconds: 0.01,
		medianSeconds: 0.14,
		runs: 10,
		command: "app --version",
		baselineMeanSeconds: 0.2,
		machine: "laptop-arm64",
	},
} satisfies Record<string, EvidenceResult>;
export function observation(
	result: EvidenceResult,
	overrides: Partial<Observation> = {},
): Observation {
	return {
		schemaVersion: 1,
		observationId: `obs-${result.__typename}`,
		manifestId: "services/api",
		section: "TESTS" as Observation["section"],
		collectorId: "unit",
		integration: "pytest",
		workspaceId: "workspace-a",
		environment: "local",
		scope: "default",
		planRevision: "plan-1",
		collectorRevision: "collector-1",
		runId: "run-1",
		attempt: 1,
		observedAt: "2026-09-17T10:00:00Z",
		sourceUpdatedAt: null,
		startedAt: "2026-09-17T09:59:57Z",
		recordedAt: "2026-09-17T10:00:01Z",
		inputFingerprint: "inputs-1",
		commitSha: "abc123456789",
		branch: "feature/cards",
		dirty: true,
		toolVersion: "1.0",
		upstreamFingerprint: null,
		policyRevision: null,
		result,
		...overrides,
	};
}
export function evidence(
	result: EvidenceResult | null = results.tests,
	overrides: Partial<CollectorEvidence> = {},
): CollectorEvidence {
	const base = {
		manifestId: "services/api",
		section: "TESTS" as CollectorEvidence["section"],
		collectorId: "unit",
		integration: "pytest",
		workspaceId: "workspace-a",
		state: "READY" as CollectorEvidence["state"],
		message: null,
		updatedAt: "2026-09-17T10:00:01Z",
		freshnessSeconds: 3600,
		stale: false,
		outdated: false,
		...overrides,
	};
	return {
		...base,
		latest: result
			? observation(result, {
					manifestId: base.manifestId,
					section: base.section,
					collectorId: base.collectorId,
					integration: base.integration,
					workspaceId: base.workspaceId,
				})
			: null,
		...overrides,
	};
}
export function dependencies(): CollectorEvidence[] {
	return [
		evidence(results.inventory, {
			section: "DEPENDENCIES" as CollectorEvidence["section"],
			collectorId: "sbom",
			integration: "syft_inventory",
		}),
		evidence(results.licenses, {
			section: "DEPENDENCIES" as CollectorEvidence["section"],
			collectorId: "license-policy",
			integration: "grant_license",
		}),
		evidence(results.vulnerabilities, {
			section: "DEPENDENCIES" as CollectorEvidence["section"],
			collectorId: "vulnerabilities",
			integration: "grype_scan",
		}),
	];
}
