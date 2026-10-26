import { expect, it } from "vite-plus/test";
import { getCardSurfaceLayout } from "./cardSurfaceLayout";

it("increases actual layout width for focus and reduces it for comparison and small viewports", () => {
	const docked = getCardSurfaceLayout(2.8, 1.8, 1400, 900, 0);
	const focused = getCardSurfaceLayout(2.8, 1.8, 1400, 900, 1);
	const compared = getCardSurfaceLayout(2.8, 1.8, 1400, 900, 2);
	const mobile = getCardSurfaceLayout(2.8, 1.8, 390, 844, 1);
	expect(docked.pixelWidth).toBe(420);
	expect(focused.pixelWidth).toBeGreaterThan(640);
	expect(compared.pixelWidth).toBeLessThan(focused.pixelWidth);
	expect(mobile.pixelWidth).toBeLessThan(640);
});

it("keeps the HTML aligned with the physical face at every layout size", () => {
	for (const count of [0, 1, 2, 5, 20]) {
		const layout = getCardSurfaceLayout(2.8, 1.8, 1400, 900, count);
		expect((layout.pixelWidth * layout.scale * 10) / 400).toBeCloseTo(2.8 - 0.12);
		expect((layout.pixelHeight * layout.scale * 10) / 400).toBeCloseTo(1.8 - 0.12);
	}
});
