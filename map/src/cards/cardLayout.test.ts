import { expect, it } from "vitest";
import { getCardLayout } from "./cardLayout";

it("gives all six evidence cards, overview, and diagrams distinct slots", () => {
	const layout = getCardLayout(
		Array.from({ length: 8 }, (_, index) => ({ components: [String(index)] })),
	);
	expect(new Set(layout.map((card) => card.zOffset)).size).toBe(8);
});
it("keeps toolbar positions but spaces only declared cards", () => {
	const layout = getCardLayout([
		{ components: ["Info"] },
		{ components: [] },
		{ components: [] },
		{ components: ["Tests"] },
		{ components: [] },
	]);
	expect(layout).toHaveLength(5);
	expect(layout[0].zOffset).toBe(-0.05);
	expect(layout[3].zOffset).toBe(0.05);
});
