// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vite-plus/test";
import { CardFocusView } from "./CardFocusView";

const focus = vi.hoisted(() => ({
	cards: [
		{
			id: "alpha",
			blockId: "Alpha",
			label: "Info",
			color: "red",
		},
		{
			id: "beta",
			blockId: "Beta",
			label: "Info",
			color: "blue",
		},
	],
	focusedIds: ["alpha"],
	expandedId: null as string | null,
	addCard: vi.fn(),
	removeCard: vi.fn(),
	expandCard: vi.fn(),
	closeFocus: vi.fn(),
}));
vi.mock("./CardFocusContext", () => ({ useCardFocus: () => focus }));

beforeEach(() => {
	focus.focusedIds = ["alpha"];
	focus.expandedId = null;
	vi.clearAllMocks();
});
afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
});

it("shows only non-modal controls and searches cards not already selected", () => {
	const { container } = render(<CardFocusView />);
	expect(container.querySelector("dialog, canvas, article")).toBeNull();
	expect(screen.getByRole("list", { name: "Focused cards" }).children).toHaveLength(1);
	fireEvent.click(screen.getByRole("button", { name: "Add card" }));
	const search = screen.getByRole("searchbox", { name: "Find a card" });
	expect(document.activeElement).toBe(search);
	fireEvent.change(search, { target: { value: "beta" } });
	fireEvent.click(screen.getByRole("button", { name: "Beta — Info" }));
	expect(focus.addCard).toHaveBeenCalledWith("beta");
	expect(focus.expandCard).toHaveBeenCalledWith(null);
	fireEvent.change(search, { target: { value: "alpha" } });
	expect(screen.getByText("No matching cards.")).toBeDefined();
});

it("expands one comparison card and Escape returns to the group before leaving", () => {
	focus.focusedIds = ["alpha", "beta"];
	const { rerender } = render(<CardFocusView />);
	expect(screen.getByRole("list", { name: "Focused cards" }).children).toHaveLength(2);
	fireEvent.click(screen.getByRole("button", { name: "Expand Beta — Info" }));
	expect(focus.expandCard).toHaveBeenCalledWith("beta");
	focus.expandedId = "beta";
	rerender(<CardFocusView />);
	expect(document.activeElement).toBe(screen.getByRole("button", { name: "Back to group" }));
	expect(screen.getByRole("list", { name: "Focused cards" }).children).toHaveLength(2);
	fireEvent.keyDown(window, { key: "Escape" });
	expect(focus.expandCard).toHaveBeenLastCalledWith(null);
	expect(focus.closeFocus).not.toHaveBeenCalled();
	focus.expandedId = null;
	rerender(<CardFocusView />);
	expect(document.activeElement).toBe(screen.getByRole("button", { name: "Expand Beta — Info" }));
	fireEvent.keyDown(window, { key: "Escape" });
	expect(focus.closeFocus).toHaveBeenCalledOnce();
});

it("removes cards and restores the previous keyboard focus on exit", () => {
	const trigger = document.createElement("button");
	document.body.append(trigger);
	trigger.focus();
	const { unmount } = render(<CardFocusView />);
	fireEvent.click(screen.getByRole("button", { name: "Remove Alpha — Info from focus" }));
	expect(focus.removeCard).toHaveBeenCalledWith("alpha");
	unmount();
	expect(document.activeElement).toBe(trigger);
	trigger.remove();
});

it("Escape closes the picker before leaving focus mode", () => {
	render(<CardFocusView />);
	fireEvent.click(screen.getByRole("button", { name: "Add card" }));
	fireEvent.keyDown(window, { key: "Escape" });
	expect(screen.queryByRole("searchbox")).toBeNull();
	expect(focus.closeFocus).not.toHaveBeenCalled();
	expect(document.activeElement).toBe(screen.getByRole("button", { name: "Add card" }));
	fireEvent.keyDown(window, { key: "Escape" });
	expect(focus.closeFocus).toHaveBeenCalledOnce();
});
