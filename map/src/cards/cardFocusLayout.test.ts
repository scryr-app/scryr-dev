import { describe, expect, it } from "vite-plus/test";
import { getCardFocusLayout } from "./cardFocusLayout";

it("centers a single card at the largest size allowed by the margins", () => {
	const [card] = getCardFocusLayout(1200, 800, 1, 1.5);
	expect(card).toEqual({ x: 0, y: 0, width: 1008, height: 672 });
	const [wide] = getCardFocusLayout(1200, 800, 1, 3);
	expect(wide.width).toBe(1080);
	expect(wide.height).toBe(360);
});

it("places two cards side by side in landscape and stacks them in portrait", () => {
	const landscape = getCardFocusLayout(1200, 800, 2, 1.5);
	expect(landscape[0].y).toBe(landscape[1].y);
	expect(landscape[0].x).toBeLessThan(landscape[1].x);
	const portrait = getCardFocusLayout(390, 844, 2, 1.5);
	expect(portrait[0].x).toBe(portrait[1].x);
	expect(portrait[0].y).toBeGreaterThan(portrait[1].y);
});

it("centers the final incomplete row", () => {
	const cards = getCardFocusLayout(1200, 800, 5, 1.5);
	const lastRow = cards.filter((card) => card.y === cards.at(-1)?.y);
	expect(lastRow.reduce((sum, card) => sum + card.x, 0)).toBeCloseTo(0);
});

describe.each([
	[1200, 800],
	[390, 844],
	[320, 200],
	[1, 1],
	[4000, 100],
	[100, 4000],
])("viewport %i × %i", (viewportWidth, viewportHeight) => {
	it.each([1, 2, 3, 5, 12, 100])("fits %i cards without overlap", (count) => {
		for (const aspect of [0.1, 1, 1.5, 10]) {
			const cards = getCardFocusLayout(viewportWidth, viewportHeight, count, aspect);
			expect(cards).toHaveLength(count);
			for (const [index, card] of cards.entries()) {
				expect(card.width).toBeGreaterThan(0);
				expect(card.height).toBeGreaterThan(0);
				expect(card.width / card.height).toBeCloseTo(aspect);
				expect(Math.abs(card.x) + card.width / 2).toBeLessThanOrEqual(viewportWidth * 0.45 + 1e-8);
				expect(Math.abs(card.y) + card.height / 2).toBeLessThanOrEqual(viewportHeight / 2);
				for (const other of cards.slice(index + 1)) {
					expect(
						Math.abs(card.x - other.x) >= (card.width + other.width) / 2 - 1e-8 ||
							Math.abs(card.y - other.y) >= (card.height + other.height) / 2 - 1e-8,
					).toBe(true);
				}
			}
		}
	});
});

it("returns no rectangles for empty or invalid input", () => {
	for (const args of [
		[1200, 800, 0, 1.5],
		[0, 800, 1, 1.5],
		[1200, 800, -1, 1.5],
		[1200, 800, 1, 0],
		[NaN, 800, 1, 1.5],
		[1200, 800, Infinity, 1.5],
	]) {
		expect(getCardFocusLayout(args[0], args[1], args[2], args[3])).toEqual([]);
	}
});
