// @vitest-environment jsdom
import { act, renderHook } from "@testing-library/react";
import { expect, it } from "vitest";
import { MapTrayProvider, useMapTray } from "./MapTrayContext";

it("docks a clicked card only on its own block and lets the toolbar reset all blocks", () => {
	const { result } = renderHook(() => useMapTray(), {
		wrapper: MapTrayProvider,
	});

	act(() => result.current.selectCardForBlock("API", 2));
	expect(result.current.getActiveCardIndex("API")).toBe(2);
	expect(result.current.getActiveCardIndex("Database")).toBe(0);
	expect(result.current.activeCardIndex).toBe(0);

	act(() => result.current.selectCardForBlock("Database", 3));
	expect(result.current.getActiveCardIndex("API")).toBe(2);
	expect(result.current.getActiveCardIndex("Database")).toBe(3);

	act(() => result.current.toggleCard(1));
	expect(result.current.getActiveCardIndex("API")).toBe(1);
	expect(result.current.getActiveCardIndex("Database")).toBe(1);
	act(() => result.current.toggleCard(1));
	expect(result.current.getActiveCardIndex("API")).toBe(0);
	expect(result.current.getActiveCardIndex("Database")).toBe(0);
});

it("keeps overview selectable and accepts arbitrary block names", () => {
	const { result } = renderHook(() => useMapTray(), {
		wrapper: MapTrayProvider,
	});
	act(() => result.current.toggleCard(2));
	act(() => result.current.selectCardForBlock("__proto__", 0));
	expect(result.current.getActiveCardIndex("__proto__")).toBe(0);
	expect(result.current.getActiveCardIndex("toString")).toBe(2);
});
