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
			readingContent: "Full alpha description",
		},
		{
			id: "beta",
			blockId: "Beta",
			label: "Info",
			color: "blue",
			readingContent: "Full beta description",
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
vi.mock("@react-three/fiber", () => ({ Canvas: () => null }));

beforeEach(() => {
	focus.focusedIds = ["alpha"];
	focus.expandedId = null;
	vi.clearAllMocks();
	HTMLDialogElement.prototype.showModal = function () {
		this.setAttribute("open", "");
	};
	HTMLDialogElement.prototype.close = function () {
		this.removeAttribute("open");
	};
	vi.stubGlobal(
		"ResizeObserver",
		class {
			observe() {}
			disconnect() {}
		},
	);
});
afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
});

it("shows one complete reading card and searches only cards not already selected", () => {
	render(<CardFocusView />);
	expect(screen.getAllByRole("article")).toHaveLength(1);
	expect(screen.getByText("Full alpha description")).toBeDefined();
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
	expect(screen.getAllByRole("article")).toHaveLength(2);
	fireEvent.click(screen.getByRole("button", { name: "Expand Beta — Info" }));
	expect(focus.expandCard).toHaveBeenCalledWith("beta");
	focus.expandedId = "beta";
	rerender(<CardFocusView />);
	expect(document.activeElement).toBe(screen.getByRole("button", { name: "Back to group" }));
	expect(screen.getAllByRole("article")).toHaveLength(1);
	fireEvent(screen.getByRole("dialog"), new Event("cancel", { bubbles: false, cancelable: true }));
	expect(focus.expandCard).toHaveBeenLastCalledWith(null);
	expect(focus.closeFocus).not.toHaveBeenCalled();
	focus.expandedId = null;
	rerender(<CardFocusView />);
	expect(document.activeElement).toBe(screen.getByRole("button", { name: "Expand Beta — Info" }));
	fireEvent(screen.getByRole("dialog"), new Event("cancel", { bubbles: false, cancelable: true }));
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
