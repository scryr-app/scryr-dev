// @vitest-environment jsdom
import {
	act,
	cleanup,
	fireEvent,
	render,
	screen,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { OpenSourcePrompt } from "./OpenSourcePrompt";

beforeEach(() => {
	vi.useFakeTimers();
	sessionStorage.clear();
});
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
	vi.useRealTimers();
});
const advance = (ms: number) => act(() => vi.advanceTimersByTime(ms));
it("appears after 20 seconds and keeps dismissal across remounts", () => {
	const view = render(<OpenSourcePrompt />);
	advance(19_999);
	expect(screen.queryByRole("complementary")).toBeNull();
	advance(1);
	expect(screen.getByRole("link").getAttribute("href")).toBe(
		"https://scryr.dev/getting-started/#install",
	);
	fireEvent.click(
		screen.getByRole("button", { name: "Dismiss install prompt" }),
	);
	view.unmount();
	render(<OpenSourcePrompt />);
	advance(20_000);
	expect(screen.queryByRole("complementary")).toBeNull();
});
it("counts visible app time and pauses in the background", () => {
	let visibility = "visible";
	vi.spyOn(document, "visibilityState", "get").mockImplementation(
		() => visibility as DocumentVisibilityState,
	);
	render(<OpenSourcePrompt />);
	advance(10_000);
	visibility = "hidden";
	fireEvent(document, new Event("visibilitychange"));
	advance(60_000);
	expect(screen.queryByRole("complementary")).toBeNull();
	visibility = "visible";
	fireEvent(document, new Event("visibilitychange"));
	advance(10_000);
	expect(screen.getByRole("complementary")).toBeTruthy();
});
it("cleans up the timer on unmount", () => {
	const view = render(<OpenSourcePrompt />);
	view.unmount();
	expect(vi.getTimerCount()).toBe(0);
});
