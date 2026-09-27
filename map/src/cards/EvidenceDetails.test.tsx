// @vitest-environment jsdom
import {
	cleanup,
	fireEvent,
	render,
	screen,
	within,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { EvidenceDetailsView } from "./EvidenceDetails";
import {
	EvidenceResultDetails,
	matchingInventory,
} from "./EvidenceResultDetails";
import type { Observation } from "./evidence";
import { collectorKey } from "./evidence";
import {
	dependencies,
	evidence,
	observation,
	results,
} from "./evidenceFixtures";

const history = vi.hoisted(() => ({
	observations: new Map<string, Observation>(),
	detailCalls: [] as Array<{
		observationId: string;
		workspaceId: string;
		offset: number;
		limit: number;
		search: string;
		filter: string;
	}>,
	packageCalls: [] as Array<{ observationId: string; ids: string[] }>,
	data: { evidenceHistory: [] as unknown[] },
	isLoading: false,
	error: null,
	refetch: vi.fn(),
}));
vi.mock("@/graphql/generated", async (original) => ({
	...(await original<object>()),
	useGetEvidenceHistoryQuery: () => history,
	useGetEvidenceObservationQuery: (variables: {
		observationId: string;
		workspaceId: string;
		offset: number;
		limit: number;
		search: string;
		filter: string;
	}) => {
		history.detailCalls.push(variables);
		const selected = history.observations.get(variables.observationId);
		if (!selected)
			return {
				data: { evidenceObservation: null },
				isLoading: false,
				isFetching: false,
			};
		const result = selected.result;
		const matches = (item: object, category: string) =>
			(!variables.filter || category === variables.filter) &&
			JSON.stringify(item)
				.toLowerCase()
				.includes(variables.search.toLowerCase());
		let paged = result;
		if (result.__typename === "InventoryResult") {
			const rows = result.packages.filter((item) =>
				matches(item, item.ecosystem),
			);
			paged = {
				...result,
				packages: rows.slice(
					variables.offset,
					variables.offset + variables.limit,
				),
				matchingPackages: rows.length,
			} as typeof result;
		} else if (result.__typename === "LicenseResult") {
			const rows = result.items.filter((item) => matches(item, item.decision));
			paged = {
				...result,
				items: rows.slice(variables.offset, variables.offset + variables.limit),
				matchingItems: rows.length,
			} as typeof result;
		} else if (result.__typename === "VulnerabilityResult") {
			const rows = result.items.filter((item) => matches(item, item.severity));
			paged = {
				...result,
				items: rows.slice(variables.offset, variables.offset + variables.limit),
				matchingFindings: rows.length,
			} as typeof result;
		}
		return {
			data: { evidenceObservation: { ...selected, result: paged } },
			isLoading: false,
			isFetching: false,
			error: null,
		};
	},
	useGetInventoryPackagesQuery: (
		variables: { observationId: string; ids: string[] },
		options: { enabled: boolean },
	) => {
		if (!options.enabled) return {};
		history.packageCalls.push(variables);
		const selected = history.observations.get(variables.observationId);
		if (selected?.result.__typename !== "InventoryResult") return {};
		return {
			data: {
				evidenceObservation: {
					...selected,
					result: {
						...selected.result,
						packages: selected.result.packages.filter((pkg) =>
							variables.ids.includes(pkg.id),
						),
					},
				},
			},
		};
	},
}));
beforeEach(() => {
	HTMLDialogElement.prototype.showModal = function () {
		this.setAttribute("open", "");
	};
	HTMLDialogElement.prototype.close = function () {
		this.removeAttribute("open");
	};
	history.data.evidenceHistory = [];
	history.observations.clear();
	history.detailCalls = [];
	history.packageCalls = [];
	for (const collector of dependencies())
		if (collector.latest)
			history.observations.set(
				collector.latest.observationId,
				collector.latest,
			);
});
afterEach(cleanup);
it("gives inventory, license evidence/policy, and vulnerability findings equal detail panels", () => {
	const collectors = dependencies();
	render(
		<EvidenceDetailsView
			section="dependencies"
			collectors={collectors}
			selectedId={collectorKey(collectors[0])}
			onClose={vi.fn()}
		/>,
	);
	expect(screen.getByRole("dialog", { name: "Dependencies" })).toBeDefined();
	expect(screen.getByText("sample-lib")).toBeDefined();
	fireEvent.click(screen.getByRole("button", { name: /^licenses/i }));
	expect(screen.getByText(/Policy has not been evaluated/)).toBeDefined();
	fireEvent.change(screen.getByRole("combobox", { name: "Collector" }), {
		target: { value: collectorKey(collectors[1]) },
	});
	expect(screen.getByText("Missing license evidence")).toBeDefined();
	fireEvent.change(screen.getByRole("combobox", { name: "Decision" }), {
		target: { value: "review" },
	});
	expect(screen.queryByText("Allowed SPDX expression")).toBeNull();
	fireEvent.click(screen.getByRole("button", { name: /^vulnerabilities/i }));
	expect(
		screen.getByRole("link", { name: "CVE-2026-1234" }).getAttribute("href"),
	).toBe("https://example.com/CVE-2026-1234");
	expect(screen.getByText("sample-lib@1.0 (npm)")).toBeDefined();
	expect(screen.getByText("1.1")).toBeDefined();
	expect(history.packageCalls.at(-1)).toMatchObject({
		observationId: collectors[0].latest?.observationId,
		ids: ["pkg-a"],
	});
});
it("does not join dependency package names across mismatched inventory or workspace provenance", () => {
	const collectors = dependencies();
	const scan = observation({
		...results.vulnerabilities,
		inventoryHash: "other-inventory",
	});
	expect(matchingInventory(scan, collectors)).toBeUndefined();
	expect(
		matchingInventory(
			observation(results.vulnerabilities, { workspaceId: "other-laptop" }),
			collectors,
		),
	).toBeUndefined();
	render(<EvidenceResultDetails observation={scan} collectors={collectors} />);
	expect(screen.getByText(/Matching inventory unavailable/)).toBeDefined();
	expect(screen.getByText("pkg-a")).toBeDefined();
	expect(screen.queryByText("sample-lib@1.0 (npm)")).toBeNull();
});
it("shows a missing scanner as unavailable, and absent license tools as unconfigured", () => {
	const scanner = evidence(null, {
		section: "DEPENDENCIES" as ReturnType<typeof evidence>["section"],
		integration: "grype_scan",
		collectorId: "security",
		state: "MISSING_TOOL" as ReturnType<typeof evidence>["state"],
		message: "grype is not installed",
	});
	render(
		<EvidenceDetailsView
			section="dependencies"
			collectors={[scanner]}
			selectedId={collectorKey(scanner)}
			onClose={vi.fn()}
		/>,
	);
	expect(screen.getByText("Missing tool")).toBeDefined();
	expect(screen.getByText("grype is not installed")).toBeDefined();
	expect(screen.getByText("scryr collect doctor")).toBeDefined();
	expect(screen.getByText(/scryr collect run --manifest/)).toBeDefined();
	fireEvent.click(screen.getByRole("button", { name: /^licenses/i }));
	expect(screen.getByText(/No licenses collector configured/)).toBeDefined();
});
it("keeps stale failed test evidence readable with exact input provenance and history", () => {
	const collector = evidence(results.tests, {
		state: "ERROR" as ReturnType<typeof evidence>["state"],
		message: "Timed out",
		stale: true,
		outdated: true,
	});
	history.data.evidenceHistory = [
		observation(
			{ ...results.tests, passing: 47 },
			{ observationId: "old", observedAt: "2026-09-16T09:00:00Z" },
		),
	];
	render(
		<EvidenceDetailsView
			section="tests"
			collectors={[collector]}
			selectedId={collectorKey(collector)}
			onClose={vi.fn()}
		/>,
	);
	expect(screen.getByText("Collection failed · Outdated inputs")).toBeDefined();
	expect(
		screen.getByText(/Inputs or the collector definition changed/),
	).toBeDefined();
	expect(screen.getByText(/48 passed, 1 failed/)).toBeDefined();
	const provenance = screen
		.getByText("Source and provenance")
		.closest("details");
	if (!provenance) throw new Error("Missing provenance");
	fireEvent.click(within(provenance).getByText("Source and provenance"));
	expect(within(provenance).getByText("inputs-1")).toBeDefined();
	const historyElement = screen.getByText("Run history").closest("details");
	if (!historyElement) throw new Error("Missing history");
	historyElement.open = true;
	fireEvent(historyElement, new Event("toggle"));
	fireEvent.click(screen.getByRole("button", { name: /47 passed/ }));
	expect(screen.getByText(/Historical observation/)).toBeDefined();
	fireEvent.click(screen.getByRole("button", { name: "Return to latest" }));
	expect(screen.queryByText(/Historical observation/)).toBeNull();
});
it("filters and paginates large inventories without losing unknown licenses", () => {
	const packages = Array.from({ length: 55 }, (_, index) => ({
		...results.inventory.packages[1],
		id: String(index),
		name: `package-${index}`,
	}));
	render(
		<EvidenceResultDetails
			observation={observation({ ...results.inventory, packages })}
			collectors={[]}
		/>,
	);
	expect(screen.getByText("55 results · Page 1 of 3")).toBeDefined();
	fireEvent.click(screen.getByRole("button", { name: "Next results" }));
	expect(screen.getByText("package-25")).toBeDefined();
	fireEvent.change(screen.getByRole("textbox", { name: "Filter results" }), {
		target: { value: "package-54" },
	});
	expect(screen.getByText("1 results · Page 1 of 1")).toBeDefined();
	expect(screen.getByText("Unknown")).toBeDefined();
});
it("shows incomplete empty scans explicitly and closes on Escape without changing drafts", () => {
	const close = vi.fn();
	const collector = evidence(
		{ ...results.vulnerabilities, complete: false, items: [] },
		{
			integration: "grype_scan",
			section: "DEPENDENCIES" as ReturnType<typeof evidence>["section"],
		},
	);
	if (collector.latest)
		history.observations.set(collector.latest.observationId, collector.latest);
	render(
		<EvidenceDetailsView
			section="dependencies"
			collectors={[collector]}
			selectedId={collectorKey(collector)}
			onClose={close}
		/>,
	);
	expect(screen.getByRole("status").textContent).toContain(
		"do not establish a clean state",
	);
	fireEvent(
		screen.getByRole("dialog"),
		new Event("cancel", { cancelable: true }),
	);
	expect(close).toHaveBeenCalledOnce();
});

it("requests immutable dependency pages and performs filters on the server without loading the whole SBOM", () => {
	const collector = dependencies()[0];
	if (!collector.latest) throw new Error("Missing inventory fixture");
	const packages = Array.from({ length: 55 }, (_, index) => ({
		...results.inventory.packages[1],
		id: String(index),
		name: `package-${index}`,
	}));
	history.observations.set(collector.latest.observationId, {
		...collector.latest,
		result: {
			...results.inventory,
			packages,
			totalPackages: 55,
			unknownLicensePackages: 55,
		},
	});
	render(
		<EvidenceDetailsView
			section="dependencies"
			collectors={[collector]}
			selectedId={collectorKey(collector)}
			onClose={vi.fn()}
		/>,
	);
	expect(history.detailCalls.at(-1)).toMatchObject({
		observationId: collector.latest.observationId,
		workspaceId: "workspace-a",
		limit: 25,
		offset: 0,
	});
	expect(screen.queryByText("package-25")).toBeNull();
	fireEvent.click(screen.getByRole("button", { name: "Next results" }));
	expect(history.detailCalls.at(-1)).toMatchObject({ offset: 25, limit: 25 });
	expect(screen.getByText("package-25")).toBeDefined();
	fireEvent.change(screen.getByRole("textbox", { name: "Filter results" }), {
		target: { value: "package-54" },
	});
	expect(history.detailCalls.at(-1)).toMatchObject({
		offset: 0,
		search: "package-54",
	});
	expect(screen.getByText("1 results · Page 1 of 1")).toBeDefined();
});

it("never fetches package names from a different inventory when opening a finding page", () => {
	const collectors = dependencies();
	if (!collectors[0].latest) throw new Error("Missing inventory fixture");
	collectors[0] = {
		...collectors[0],
		latest: {
			...collectors[0].latest,
			result: { ...results.inventory, artifactHash: "new-inventory" },
		},
	};
	render(
		<EvidenceDetailsView
			section="dependencies"
			collectors={collectors}
			selectedId={collectorKey(collectors[2])}
			onClose={vi.fn()}
		/>,
	);
	expect(history.packageCalls).toHaveLength(0);
	expect(screen.getByText(/Matching inventory unavailable/)).toBeDefined();
	expect(screen.getByText("pkg-a")).toBeDefined();
});
