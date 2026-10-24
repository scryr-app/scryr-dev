// @vitest-environment jsdom
import { expect, it, vi } from "vite-plus/test";
import { createHtmlCardTexture, supportsNativeHtml } from "./htmlCardNative";

it("requires the texture upload API as well as canvas paint support", () => {
	const canvas = document.createElement("canvas");
	expect(supportsNativeHtml(canvas, {})).toBe(false);
	Object.assign(canvas, { requestPaint: vi.fn() });
	expect(supportsNativeHtml(canvas, {})).toBe(false);
	expect(supportsNativeHtml(canvas, { texElementImage2D: vi.fn() })).toBe(true);
});

it("updates on paint and cleans up without replacing another canvas paint handler", () => {
	const canvas = document.createElement("canvas");
	const requestPaint = vi.fn();
	const existing = vi.fn();
	Object.assign(canvas, { requestPaint, onpaint: existing });
	const host = document.createElement("div");
	const onPaint = vi.fn();
	const resource = createHtmlCardTexture(canvas, host, onPaint);
	expect(host.parentElement).toBe(canvas);
	expect(requestPaint).toHaveBeenCalledOnce();
	const version = resource.texture.version;
	canvas.dispatchEvent(new Event("paint"));
	expect(resource.texture.version).toBeGreaterThan(version);
	expect(onPaint).toHaveBeenCalledOnce();
	resource.dispose();
	canvas.dispatchEvent(new Event("paint"));
	expect(onPaint).toHaveBeenCalledOnce();
	expect(host.parentElement).toBeNull();
	expect((canvas as HTMLCanvasElement & { onpaint: unknown }).onpaint).toBe(existing);
	expect(canvas.hasAttribute("layoutsubtree")).toBe(false);
});
