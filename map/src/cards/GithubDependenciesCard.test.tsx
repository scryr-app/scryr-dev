// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { GithubDependenciesCard } from "./GithubDependenciesCard";
import { githubDependenciesData } from "./githubDependenciesData";

vi.mock("@react-three/uikit", () => ({
	Container: ({
		children,
		onClick,
	}: {
		children: ReactNode;
		onClick?: () => void;
	}) =>
		onClick ? (
			<button type="button" onClick={onClick}>
				{children}
			</button>
		) : (
			<div>{children}</div>
		),
	Text: ({ children }: { children: ReactNode }) => <span>{children}</span>,
}));
vi.mock("@/theme/theme", () => ({
	currentTheme: { cardTextColor: "white", cardMutedTextColor: "gray" },
}));
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});
const observedAt = "2026-09-17T12:00:00Z";
const now = Date.parse(observedAt) + 300_000;
const alert = {
	number: 1,
	package: "widget",
	ecosystem: "npm",
	manifestPath: "package-lock.json",
	severity: "high",
	url: "https://github.com/acme/api/security/dependabot/1",
	firstPatchedVersion: "2.1.0",
	vulnerableVersionRange: "< 2.1.0",
	ghsaId: "GHSA-abcd",
	summary: "A reported vulnerability",
};
function card({
	alerts,
	inventory,
	error,
	inventoryError,
	securityEnabled = true,
	clock = now,
	repository = "acme/api",
}: {
	alerts?: object[];
	inventory?: object;
	error?: string;
	inventoryError?: string;
	securityEnabled?: boolean;
	clock?: number;
	repository?: string;
} = {}) {
	const data = githubDependenciesData(
		{
			github: { repoUrl: `https://github.com/${repository}` },
			dependencies: {
				source: {
					provider: "github",
					inventory: true,
					security: securityEnabled,
				},
				github: {
					repository,
					inventory,
					security: alerts ? { observedAt, alerts } : undefined,
				},
			},
			providerSync: {
				github_dependencies_security: {
					lastAttemptAt: observedAt,
					lastSuccessAt: error ? observedAt : undefined,
					error,
					context: { repository },
				},
				github_dependencies_inventory: {
					error: inventoryError,
					context: { repository },
				},
			},
		},
		clock,
	);
	if (!data) throw new Error("Expected configured dependency data");
	return <GithubDependenciesCard {...data} />;
}

describe("GitHub dependency card", () => {
	it("shows repository scope and keeps missing security unknown", () => {
		render(card());
		expect(screen.getByText("Repository-wide · acme/api")).toBeTruthy();
		expect(screen.getByText("Current security unknown")).toBeTruthy();
		expect(screen.getByText("Inventory unavailable")).toBeTruthy();
		expect(screen.queryByText(/0 open alerts/)).toBeNull();
	});
	it("shows verified zero alerts even when inventory is unavailable", () => {
		render(
			card({
				alerts: [],
				inventoryError: "GitHub inventory unavailable (404)",
			}),
		);
		expect(
			screen.getByText("0 vulnerable packages · 0 open alerts"),
		).toBeTruthy();
		expect(screen.getByText("Inventory unavailable")).toBeTruthy();
		expect(
			screen.getByText(
				"Inventory collection error: GitHub inventory unavailable (404)",
			),
		).toBeTruthy();
		expect(screen.queryByText("Current security unknown")).toBeNull();
	});
	it("labels previous zero-alert results last known during a failed poll", () => {
		render(card({ alerts: [], error: "Security permission denied" }));
		expect(screen.getByText("Current security unknown")).toBeTruthy();
		expect(
			screen.getByText("Last known: 0 vulnerable packages · 0 open alerts"),
		).toBeTruthy();
		expect(
			screen.queryByText("0 vulnerable packages · 0 open alerts"),
		).toBeNull();
		expect(
			screen.getByText("Security collection error: Security permission denied"),
		).toBeTruthy();
		expect(
			screen.getByText("Security last sync: 2026-09-17 12:00:00 UTC"),
		).toBeTruthy();
	});
	it("labels stale snapshots and their observation time", () => {
		render(card({ alerts: [alert], clock: now + 3 * 60 * 60 * 1000 }));
		expect(screen.getByText("Current security unknown")).toBeTruthy();
		expect(
			screen.getByText("Security observed: 2026-09-17 12:00:00 UTC · stale"),
		).toBeTruthy();
	});
	it("shows severity, advisory details, and patched versions with safe remediation links", () => {
		const open = vi.spyOn(window, "open").mockImplementation(() => null);
		render(card({ alerts: [alert] }));
		expect(
			screen.getByText("1 vulnerable packages · 1 open alerts"),
		).toBeTruthy();
		expect(screen.getByText("Highest severity: high")).toBeTruthy();
		expect(
			screen.getByText("critical: 0 · high: 1 · medium: 0 · low: 0"),
		).toBeTruthy();
		expect(screen.getByText("First patched version: 2.1.0")).toBeTruthy();
		expect(screen.getByText("Affected: < 2.1.0")).toBeTruthy();
		fireEvent.click(screen.getByText("widget (npm) · high ↗"));
		expect(open).toHaveBeenCalledWith(
			alert.url,
			"_blank",
			"noopener,noreferrer",
		);
	});
	it("retains malicious-link alerts without making the link clickable", () => {
		const open = vi.spyOn(window, "open").mockImplementation(() => null);
		render(card({ alerts: [{ ...alert, url: "javascript:alert(1)" }] }));
		fireEvent.click(screen.getByText("widget (npm) · high"));
		expect(open).not.toHaveBeenCalled();
	});
	it("hides disabled security snapshots instead of showing zero", () => {
		render(
			card({ alerts: [alert], securityEnabled: false, error: "old error" }),
		);
		expect(screen.getByText("Security disabled")).toBeTruthy();
		expect(screen.queryByText(/open alerts/)).toBeNull();
		expect(screen.queryByText(/old error/)).toBeNull();
	});
	it("shows inventory observations and supplied counts without inventing health claims", () => {
		render(
			card({
				inventory: {
					observedAt,
					packages: [
						{ id: "1", name: "widget", version: "1.0", license: "MIT" },
					],
					directDeps: 0,
					transitiveDeps: 1,
				},
			}),
		);
		expect(screen.getByText("Packages: 1")).toBeTruthy();
		expect(screen.getByText("Direct dependencies: 0")).toBeTruthy();
		expect(screen.getByText("Transitive dependencies: 1")).toBeTruthy();
		expect(screen.getByText("widget 1.0 · MIT")).toBeTruthy();
		expect(screen.queryByText(/outdated|compliant/i)).toBeNull();
	});
});

function rows(total: number, prefix = "package") {
	return {
		alerts: Array.from({ length: total }, (_, index) => ({
			...alert,
			number: index + 1,
			package: `${prefix}-${index + 1}`,
		})),
		inventory: {
			observedAt,
			packages: Array.from({ length: total }, (_, index) => ({
				id: String(index + 1),
				name: `${prefix}-${index + 1}`,
			})),
		},
	};
}

describe("Dependency card pagination", () => {
	it("renders at most twenty rows per section while preserving complete totals and severity counts", () => {
		render(card(rows(45)));
		expect(
			screen.getByText("45 vulnerable packages · 45 open alerts"),
		).toBeTruthy();
		expect(screen.getByText("Packages: 45")).toBeTruthy();
		expect(
			screen.getByText("critical: 0 · high: 45 · medium: 0 · low: 0"),
		).toBeTruthy();
		expect(screen.getByText("Alerts: 1–20 of 45")).toBeTruthy();
		expect(screen.getByText("Packages: 1–20 of 45")).toBeTruthy();
		expect(screen.getByText("package-20")).toBeTruthy();
		expect(screen.queryByText("package-21")).toBeNull();
		expect(screen.queryByText("package-21 (npm) · high ↗")).toBeNull();
	});
	it("pages alerts independently of packages and supports Previous and the last partial page", () => {
		render(card(rows(45)));
		fireEvent.click(screen.getByText("Next alerts"));
		expect(screen.getByText("Alerts: 21–40 of 45")).toBeTruthy();
		expect(screen.queryByText("package-1 (npm) · high ↗")).toBeNull();
		expect(screen.getByText("package-1")).toBeTruthy();
		fireEvent.click(screen.getByText("Next alerts"));
		expect(screen.getByText("Alerts: 41–45 of 45")).toBeTruthy();
		expect(screen.getByText("package-45 (npm) · high ↗")).toBeTruthy();
		fireEvent.click(screen.getByText("Next alerts"));
		expect(screen.getByText("Alerts: 41–45 of 45")).toBeTruthy();
		fireEvent.click(screen.getByText("Previous alerts"));
		expect(screen.getByText("Alerts: 21–40 of 45")).toBeTruthy();
		fireEvent.click(screen.getByText("Next packages"));
		expect(screen.getByText("Packages: 21–40 of 45")).toBeTruthy();
		expect(screen.queryByText("package-1")).toBeNull();
	});
	it("clamps both pages as refreshed snapshots shrink and keeps them clamped on growth", () => {
		const view = render(card(rows(45)));
		for (const label of ["alerts", "packages"]) {
			fireEvent.click(screen.getByText(`Next ${label}`));
			fireEvent.click(screen.getByText(`Next ${label}`));
		}
		view.rerender(card(rows(2)));
		expect(screen.getByText("Alerts: 1–2 of 2")).toBeTruthy();
		expect(screen.getByText("Packages: 1–2 of 2")).toBeTruthy();
		view.rerender(card(rows(45)));
		expect(screen.getByText("Alerts: 1–20 of 45")).toBeTruthy();
		expect(screen.getByText("Packages: 1–20 of 45")).toBeTruthy();
	});
	it("starts at the first page when the repository changes", () => {
		const view = render(card(rows(45)));
		fireEvent.click(screen.getByText("Next alerts"));
		fireEvent.click(screen.getByText("Next packages"));
		view.rerender(card({ ...rows(45, "new"), repository: "acme/new-api" }));
		expect(screen.getByText("Repository-wide · acme/new-api")).toBeTruthy();
		expect(screen.getByText("Alerts: 1–20 of 45")).toBeTruthy();
		expect(screen.getByText("Packages: 1–20 of 45")).toBeTruthy();
		expect(screen.getByText("new-1")).toBeTruthy();
		expect(screen.queryByText("package-21")).toBeNull();
	});
	it("shows zero ranges without navigation for complete empty snapshots", () => {
		render(card(rows(0)));
		expect(screen.getByText("Alerts: 0 of 0")).toBeTruthy();
		expect(screen.getByText("Packages: 0 of 0")).toBeTruthy();
		expect(screen.queryByText("Next alerts")).toBeNull();
		expect(screen.queryByText("Previous packages")).toBeNull();
	});
});
