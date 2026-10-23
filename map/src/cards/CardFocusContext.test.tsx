// @vitest-environment jsdom
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it } from "vite-plus/test";
import { mapSelectionStore } from "@/graphql/sampleStore";
import {
	CardFocusProvider,
	type FocusCard,
	getFocusCardId,
	useCardFocus,
} from "./CardFocusContext";

const cards: FocusCard[] = ["API", "Database", "Worker"].map((blockId) => ({
	id: getFocusCardId(blockId, 0),
	blockId,
	label: "Info",
	color: "#ffffff",
}));
afterEach(cleanup);
function setup() {
	const hook = renderHook(() => useCardFocus(), { wrapper: CardFocusProvider });
	act(() => hook.result.current.registerCards(cards));
	return hook;
}
it("starts with one card, deduplicates additions, and starts fresh when focused again", () => {
	const { result } = setup();
	act(() => result.current.focusCard(cards[0].id));
	expect(result.current.focusedIds).toEqual([cards[0].id]);
	act(() => {
		result.current.addCard(cards[1].id);
		result.current.addCard(cards[1].id);
		result.current.addCard("missing");
	});
	expect(result.current.focusedIds).toEqual([cards[0].id, cards[1].id]);
	act(() => result.current.focusCard(cards[2].id));
	expect(result.current.focusedIds).toEqual([cards[2].id]);
});
it("expands temporarily without losing the comparison group, and closes when the last card is removed", () => {
	const { result } = setup();
	act(() => {
		result.current.focusCard(cards[0].id);
		result.current.addCard(cards[1].id);
		result.current.expandCard(cards[1].id);
	});
	expect(result.current.expandedId).toBe(cards[1].id);
	expect(result.current.focusedIds).toHaveLength(2);
	act(() => result.current.expandCard(null));
	expect(result.current.focusedIds).toHaveLength(2);
	act(() => {
		result.current.removeCard(cards[0].id);
		result.current.removeCard(cards[1].id);
	});
	expect(result.current.focusedIds).toEqual([]);
	expect(result.current.expandedId).toBeNull();
});
it("refreshes metadata and removes unavailable cards after an edit", () => {
	const { result } = setup();
	act(() => {
		result.current.focusCard(cards[0].id);
		result.current.addCard(cards[1].id);
		result.current.expandCard(cards[1].id);
	});
	act(() => result.current.registerCards([{ ...cards[0], label: "updated" }]));
	expect(result.current.focusedIds).toEqual([cards[0].id]);
	expect(result.current.expandedId).toBeNull();
	expect(result.current.cards[0].label).toBe("updated");
});
it("clears focus and its catalog when changing diagrams", () => {
	const { result } = setup();
	act(() => result.current.focusCard(cards[0].id));
	act(() => mapSelectionStore.set({ id: "another-map", key: "" }));
	expect(result.current.focusedIds).toEqual([]);
	expect(result.current.cards).toEqual([]);
});
it("closes without discarding the catalog, and never expands a card outside the group", () => {
	const { result } = setup();
	act(() => {
		result.current.focusCard(cards[0].id);
		result.current.expandCard(cards[1].id);
	});
	expect(result.current.expandedId).toBeNull();
	act(() => result.current.closeFocus());
	expect(result.current.focusedIds).toEqual([]);
	expect(result.current.cards).toHaveLength(3);
});

it("waits for a valid card before ending selection and opening single-card focus", () => {
	const { result } = setup();
	act(() => result.current.toggleSelection());
	expect(result.current.isSelecting).toBe(true);
	expect(result.current.focusedIds).toEqual([]);
	act(() => result.current.focusCard("missing"));
	expect(result.current.isSelecting).toBe(true);
	act(() => result.current.focusCard(cards[1].id));
	expect(result.current.isSelecting).toBe(false);
	expect(result.current.focusedIds).toEqual([cards[1].id]);
});
