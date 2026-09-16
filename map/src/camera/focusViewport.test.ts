// @vitest-environment jsdom
import { afterEach, expect, it, vi } from "vitest";
import { DEFAULT_FOCUS_VIEWPORT, getFocusViewport } from "./focusViewport";

afterEach(() => {
	document.body.replaceChildren();
});

function element(
	left: number,
	top: number,
	width: number,
	height: number,
	overlay = false,
) {
	const node = document.createElement(overlay ? "div" : "canvas");
	if (overlay) node.dataset.cameraOccluder = "";
	document.body.append(node);
	vi.spyOn(node, "getBoundingClientRect").mockReturnValue(
		new DOMRect(left, top, width, height),
	);
	return node;
}

it("fits beside the editor and above the toolbar without closing either", () => {
	const canvas = element(0, 0, 1400, 1000);
	element(16, 72, 420, 736, true);
	element(350, 920, 700, 56, true);
	const viewport = getFocusViewport(canvas);
	expect((viewport.minX + 1) * 700).toBeGreaterThan(436);
	expect((1 - viewport.minY) * 500).toBeLessThan(920);
	expect(viewport.maxX).toBeLessThan(1);
	expect(viewport.maxY).toBeLessThan(1);
});

it("recomputes free space when a floating panel moves or closes", () => {
	const canvas = element(100, 50, 1000, 800);
	const editor = element(750, 70, 320, 700, true);
	expect(getFocusViewport(canvas).maxX).toBeLessThan(0.3);
	editor.remove();
	expect(getFocusViewport(canvas)).toEqual({
		minX: expect.closeTo(-0.82),
		maxX: expect.closeTo(0.82),
		minY: expect.closeTo(-0.82),
		maxY: expect.closeTo(0.82),
	});
});

it("ignores hidden/outside overlays and handles an unmeasured canvas", () => {
	expect(getFocusViewport(null)).toEqual(DEFAULT_FOCUS_VIEWPORT);
	expect(getFocusViewport(element(0, 0, 0, 0))).toEqual(DEFAULT_FOCUS_VIEWPORT);
	element(0, 0, 0, 0, true);
	element(1500, 0, 300, 500, true);
	expect(getFocusViewport(element(0, 0, 1400, 1000))).toEqual({
		minX: expect.closeTo(-0.82),
		maxX: expect.closeTo(0.82),
		minY: expect.closeTo(-0.82),
		maxY: expect.closeTo(0.82),
	});
});
