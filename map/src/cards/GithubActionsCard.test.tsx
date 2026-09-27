// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { GithubActionsCard } from "./GithubActionsCard";
import type { WorkflowStatus } from "./githubActionsData";

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

const passing: WorkflowStatus = {
	key: "ci",
	name: "CI",
	branch: "main",
	status: "success",
	outcome: "passing",
	url: "https://github.com/acme/api/actions/runs/1",
};

describe("GitHub Actions card", () => {
	it("keeps all workflow outcomes visible and separates sync failures from build results", () => {
		render(
			<GithubActionsCard
				workflows={[
					passing,
					{
						...passing,
						key: "integration",
						name: "Integration",
						status: "failure",
						outcome: "failing",
					},
				]}
				sync={{
					lastSuccessAt: "2026-09-17T12:05:00Z",
					lastAttemptAt: "2026-09-17T12:10:00Z",
					error: "gh is unavailable",
				}}
			/>,
		);
		expect(
			screen.getByText("2 workflow branches · 1 failing · 0 pending"),
		).toBeTruthy();
		expect(screen.getByText("main · success ↗")).toBeTruthy();
		expect(screen.getByText("main · failure ↗")).toBeTruthy();
		expect(screen.getByText("Last sync: 2026-09-17 12:05:00 UTC")).toBeTruthy();
		expect(
			screen.getByText("Collection error: gh is unavailable"),
		).toBeTruthy();
		expect(
			screen.getByText(
				"Sync failed: 2026-09-17 12:10:00 UTC · showing last observations",
			),
		).toBeTruthy();
	});
	it("does not imply an empty initial sync means successful builds", () => {
		render(
			<GithubActionsCard
				workflows={[]}
				sync={{ error: "gh authentication required" }}
			/>,
		);
		expect(screen.getByText("No workflow runs observed")).toBeTruthy();
		expect(screen.getByText("No successful sync yet")).toBeTruthy();
	});
	it("makes every selected workflow accessible without truncating long lists", () => {
		const workflows = Array.from({ length: 20 }, (_, i) => ({
			...passing,
			key: `workflow-${i}`,
			name: `Workflow ${i}`,
		}));
		render(<GithubActionsCard workflows={workflows} />);
		expect(screen.getByText("Workflow 19")).toBeTruthy();
		expect(
			screen.getByText("20 workflow branches · 0 failing · 0 pending"),
		).toBeTruthy();
		expect(screen.getByText("No polling sync reported")).toBeTruthy();
	});
	it("opens the workflow run without sharing the window opener", () => {
		const open = vi.spyOn(window, "open").mockImplementation(() => null);
		render(<GithubActionsCard workflows={[passing]} />);
		fireEvent.click(screen.getByText("CI"));
		expect(open).toHaveBeenCalledWith(
			passing.url,
			"_blank",
			"noopener,noreferrer",
		);
	});
	it("does not open a window for a workflow with no safe URL", () => {
		const open = vi.spyOn(window, "open").mockImplementation(() => null);
		render(<GithubActionsCard workflows={[{ ...passing, url: undefined }]} />);
		fireEvent.click(screen.getByText("CI"));
		expect(open).not.toHaveBeenCalled();
	});
	it("preserves deployment and pipeline observations including explicit zeroes", () => {
		render(
			<GithubActionsCard
				workflows={[passing]}
				pipeline={{
					deployStatusProd: "deployed",
					deployStatusStaging: "deploying",
					pipelineDuration: 3,
					deployFrequency: 0,
					failedBuilds: 0,
				}}
			/>,
		);
		expect(screen.getByText("Production: deployed")).toBeTruthy();
		expect(screen.getByText("Staging: deploying")).toBeTruthy();
		expect(screen.getByText("Pipeline: 3 minutes")).toBeTruthy();
		expect(screen.getByText("Deployments: 0/week")).toBeTruthy();
		expect(screen.getByText("Failed builds (7 days): 0")).toBeTruthy();
	});
});
