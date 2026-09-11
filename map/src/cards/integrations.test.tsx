// @vitest-environment jsdom
import {
	act,
	cleanup,
	fireEvent,
	render,
	screen,
} from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { getCardLayout } from "./cardLayout";
import { selectActionRun } from "./IntegrationCard";
import { IntegrationSetupDialog } from "./IntegrationSetup";
import {
	type IntegrationView,
	openIntegrationSetup,
} from "./integrationCatalog";

afterEach(cleanup);
it("filters Actions data by repository, workflow, and branch", () => {
	const card: IntegrationView = {
		id: "ci",
		kind: "github_actions_pipeline",
		category: "cicd",
		title: "GitHub Actions",
		integration: {
			id: "github",
			kind: "github_actions",
			repository: "owner/api",
		},
		workflowId: 42,
		branch: "main",
	};
	const run = {
		repository: "owner/api",
		workflowId: 42,
		headBranch: "main",
		updatedAt: "2026-09-11T00:00:00Z",
		status: "completed",
		conclusion: "success",
	};
	expect(
		selectActionRun(card, [
			run,
			{ ...run, workflowId: 43, updatedAt: "2026-09-12T00:00:00Z" },
			{ ...run, repository: "owner/other" },
		]),
	).toEqual(run);
});
it("opens category-specific setup guidance and closes with Escape", () => {
	render(<IntegrationSetupDialog />);
	act(() => openIntegrationSetup("performance"));
	expect(screen.getByRole("dialog")).toBeTruthy();
	expect(screen.getByRole("heading", { name: "Grafana" })).toBeTruthy();
	expect(screen.getByRole("heading", { name: "PostHog" })).toBeTruthy();
	expect(screen.queryByRole("heading", { name: "GitHub Actions" })).toBeNull();
	expect(screen.getByText(/scryr.secrets.toml beside it/)).toBeTruthy();
	fireEvent.keyDown(window, { key: "Escape" });
	expect(screen.queryByRole("dialog")).toBeNull();
});
it("lays out more than seven categories without overlapping offsets", () => {
	const layout = getCardLayout(
		Array.from({ length: 12 }, (_, i) => ({
			id: String(i),
			components: [String(i)],
		})),
	);
	expect(new Set(layout.map((c) => c.zOffset)).size).toBe(12);
	expect(layout[0].zOffset).toBeCloseTo(-0.3);
	expect(layout[11].zOffset).toBeCloseTo(0.3);
});
