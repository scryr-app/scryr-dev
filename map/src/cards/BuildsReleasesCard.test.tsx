// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, it } from "vite-plus/test";
import {
	DeliveryProvider,
	DeliveryDeploymentStatus,
	DeliveryEligibility,
	DeliveryEvidenceState,
	DeliverySourceCategory,
	type GetBlocksQuery,
} from "@/graphql/generated";
import { BuildsReleasesCard } from "./BuildsReleasesCard";

type Block = GetBlocksQuery["blocks"][number];
const delivery: NonNullable<Block["delivery"]> = {
	kind: DeliveryProvider.Github,
	tasks: {
		collect: { limit: 20, verifyAttestations: false },
		verify: { limit: 20, verifyAttestations: true },
	},
	repository: { host: "github.com", owner: "example", name: "payments" },
	builds: { workflow: ".github/workflows/build.yml", artifactNames: [] },
	releases: { includeDrafts: false, includePrereleases: false },
	environments: [
		{
			name: "production",
			deployments: { environment: "production", task: "deploy" },
			runtime: null,
			compatibility: null,
			rollback: {
				requireArtifactAvailable: true,
				requireCompatibilityEvidence: true,
				requireVerifiedProvenance: true,
			},
		},
	],
};
const snapshot: NonNullable<Block["deliverySnapshot"]> = {
	host: "github.com",
	repository: "example/payments",
	observedAt: "2026-11-04T12:00:00Z",
	sources: [],
	builds: [],
	artifacts: [],
	releases: [],
	links: [],
	runtime: [],
	compatibility: [],
	deployments: [
		{
			id: "184",
			environment: "production",
			commitSha: "a".repeat(40),
			status: DeliveryDeploymentStatus.Success,
			statuses: [],
			createdAt: "2026-11-04T11:59:00Z",
			updatedAt: "2026-11-04T12:00:00Z",
			url: "https://github.com/example/payments/deployments",
		},
	],
};

afterEach(cleanup);

it("shows an uncollected state without inventing delivery results", () => {
	render(<BuildsReleasesCard delivery={delivery} assessments={[]} />);
	expect(screen.getByRole("status").textContent).toContain(
		"No delivery observations collected yet",
	);
	expect(screen.queryByText("Eligible under policy")).toBeNull();
});

it("does not turn a successful deployment into running-version or rollback proof", () => {
	render(<BuildsReleasesCard delivery={delivery} snapshot={snapshot} assessments={[]} />);
	const environment = screen.getByRole("region", { name: "production delivery" });
	expect(environment.textContent).toContain("success reported");
	expect(environment.textContent).toContain("Not verified");
	expect(environment.textContent).toContain("No deployment artifact link");
	expect(within(environment).getByText("Rollback").nextElementSibling?.textContent).toBe("Unknown");
	expect(screen.getByText("Not tested")).toBeDefined();
});

it("retains mixed runtime observations while marking stale evidence as unverified", () => {
	const runtime = {
		schema: "scryr.runtime.v1",
		manifestId: "payments",
		environment: "production",
		deploymentId: "184",
		configurationRevision: "config-1",
		schemaRevision: "schema-9",
		observedAt: "2026-11-03T12:00:00Z",
		workloads: [
			{ name: "api", artifactDigest: `sha256:${"a".repeat(64)}`, replicas: 3, readyReplicas: 3 },
			{ name: "api", artifactDigest: `sha256:${"b".repeat(64)}`, replicas: 2, readyReplicas: 1 },
		],
	};
	render(
		<BuildsReleasesCard
			delivery={delivery}
			snapshot={{ ...snapshot, runtime: [runtime] }}
			assessments={[
				{
					environment: "production",
					runtimeState: DeliveryEvidenceState.Stale,
					runningMatchesDeployment: null,
					candidateDeploymentId: null,
					candidateDigest: null,
					eligibility: DeliveryEligibility.Unknown,
					checks: [],
				},
			]}
		/>,
	);
	expect(screen.getByText("Not verified · stale")).toBeDefined();
	expect(screen.getByText("Runtime evidence (not current)")).toBeDefined();
	expect(screen.getByText(runtime.workloads[0].artifactDigest)).toBeDefined();
	expect(screen.getByText(runtime.workloads[1].artifactDigest)).toBeDefined();
});

it("shows source failures and rejects unsafe evidence URLs", () => {
	render(
		<BuildsReleasesCard
			delivery={delivery}
			snapshot={{
				...snapshot,
				sources: [
					{
						category: DeliverySourceCategory.Builds,
						environment: null,
						state: DeliveryEvidenceState.AccessDenied,
						message: "Actions read permission is missing",
					},
				],
				deployments: [{ ...snapshot.deployments[0], url: "javascript:alert(1)" }],
			}}
			assessments={[]}
		/>,
	);
	expect(screen.getByText("Actions read permission is missing")).toBeDefined();
	expect(screen.getByRole("region", { name: "Collection status" }).textContent).toContain(
		"access denied",
	);
	for (const link of screen.getAllByRole("link"))
		expect(link.getAttribute("href")).toMatch(/^https?:/);
});
