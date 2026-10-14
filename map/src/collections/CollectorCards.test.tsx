// @vitest-environment jsdom
import {
	act,
	cleanup,
	fireEvent,
	render,
	screen,
	within,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { GetCollectorCardsQuery } from "@/graphql/generated";
import { CollectorCards } from "./CollectorCards";

const state = vi.hoisted(() => ({
	cards: [] as GetCollectorCardsQuery["collectorCards"],
	error: null as Error | null,
	isLoading: false,
	local: true,
}));
const hook = vi.hoisted(() => vi.fn());
vi.mock("@/graphql/generated", () => ({
	useGetCollectorCardsQuery: (...args: unknown[]) => {
		hook(...args);
		return {
			data: { collectorCards: state.cards },
			error: state.error,
			isLoading: state.isLoading,
		};
	},
}));
vi.mock("@/auth/env", () => ({
	get isLocalAuthMode() {
		return state.local;
	},
}));
vi.mock("@/graphql/sampleStore", () => ({
	useSelectedMap: () => ({ id: "selected", key: "selected" }),
}));
vi.mock("@/graphql/useBlocksData", () => ({
	useBlocksData: () => ({ blocks: [{ manifestId: "api" }] }),
}));
vi.mock("@/theme/theme", () => ({
	useTheme: () => ({ console: { background: "white", text: "black" } }),
}));

const success = {
	id: "observation",
	collectorId: "deployment",
	startedAt: "2026-01-01T10:00:00Z",
	finishedAt: "2026-01-01T10:00:01Z",
	status: "success",
	exitCode: 0,
	error: null,
	data: { version: "v1", state: "degraded" },
	planRevision: "revision",
};
const card: GetCollectorCardsQuery["collectorCards"][number] = {
	id: "status",
	title: "API deployment",
	kind: "status",
	target: "api",
	collectorId: "deployment",
	environment: "production",
	staleAfterSeconds: 120,
	latest: success,
	lastSuccess: success,
	history: [success],
};
beforeEach(() => {
	state.cards = [];
	state.error = null;
	state.isLoading = false;
	state.local = true;
	hook.mockClear();
});
afterEach(() => {
	cleanup();
	vi.useRealTimers();
});
function open() {
	fireEvent.click(screen.getByRole("button", { name: /Observations/ }));
}

it("ages observations even when polling returns unchanged data", () => {
	vi.useFakeTimers();
	vi.setSystemTime(new Date("2026-01-01T10:01:59Z"));
	state.cards = [card];
	render(<CollectorCards isEditorOpen={false} onExpand={() => {}} />);
	open();
	expect(screen.getByText("Observation current")).toBeTruthy();
	act(() => vi.advanceTimersByTime(5_000));
	expect(screen.getByText("Stale observation")).toBeTruthy();
});

it("hides when the selected map has no cards and never queries in hosted mode", () => {
	state.cards = [{ ...card, target: "another-map" }];
	const { rerender } = render(
		<CollectorCards isEditorOpen={false} onExpand={() => {}} />,
	);
	expect(screen.queryByRole("complementary")).toBeNull();
	state.local = false;
	rerender(<CollectorCards isEditorOpen={false} onExpand={() => {}} />);
	expect(hook).toHaveBeenLastCalledWith(undefined, {
		enabled: false,
		refetchInterval: 5000,
	});
});

it("separates collection failure and stale last successful evidence from application status", () => {
	state.cards = [
		{
			...card,
			latest: {
				...success,
				id: "failed",
				status: "timeout",
				data: null,
				error: "Task exceeded timeout",
			},
		},
	];
	render(<CollectorCards isEditorOpen={false} onExpand={() => {}} />);
	open();
	expect(screen.getByText("Collection failed")).toBeTruthy();
	expect(screen.getByText(/Task exceeded timeout/)).toBeTruthy();
	expect(screen.getByText(/Last successful collection/).textContent).toContain(
		"Stale",
	);
	expect(screen.getByText("degraded")).toBeTruthy();
	expect(screen.getByText("v1")).toBeTruthy();
});

it("renders multiple views over the same collector, filtered to the selected map", () => {
	state.cards = [
		card,
		{ ...card, id: "table", kind: "table", title: "Deployment fields" },
		{ ...card, id: "history", kind: "history", title: "Deployment history" },
		{ ...card, id: "other", target: "other", title: "Unrelated" },
	];
	render(<CollectorCards isEditorOpen={false} onExpand={() => {}} />);
	open();
	expect(screen.getAllByRole("article")).toHaveLength(3);
	expect(within(screen.getByRole("table")).getByText("v1")).toBeTruthy();
	expect(within(screen.getByRole("list")).getByText("v1")).toBeTruthy();
	expect(screen.queryByText("Unrelated")).toBeNull();
	expect(hook).toHaveBeenCalledWith(undefined, {
		enabled: true,
		refetchInterval: 5000,
	});
});

it("shows awaiting and refresh failure states without inventing health", () => {
	state.cards = [{ ...card, latest: null, lastSuccess: null, history: [] }];
	state.error = new Error("Offline");
	render(<CollectorCards isEditorOpen={false} onExpand={() => {}} />);
	open();
	expect(screen.getByText("Awaiting first observation")).toBeTruthy();
	expect(screen.getByRole("alert").textContent).toContain("out of date");
});

it("hands space back and forth with the source editor", () => {
	state.cards = [card];
	const onExpand = vi.fn();
	const { rerender } = render(
		<CollectorCards isEditorOpen onExpand={onExpand} />,
	);
	open();
	expect(onExpand).toHaveBeenCalledOnce();
	rerender(<CollectorCards isEditorOpen={false} onExpand={onExpand} />);
	expect(screen.getByRole("article")).toBeTruthy();
	rerender(<CollectorCards isEditorOpen onExpand={onExpand} />);
	expect(screen.queryByRole("article")).toBeNull();
});
