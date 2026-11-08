import { expect, it } from "vite-plus/test";
import { getCardLayout } from "./cardLayout";

it("keeps category indexes while spacing cards across empty categories", () => {
	const slots = getCardLayout([
		{ components: ["Info"] },
		{ components: [] },
		{ components: ["Builds & releases"] },
	]);
	expect(slots).toHaveLength(3);
	expect(slots[0].zOffset).toBe(-0.1);
	expect(slots[2].zOffset).toBe(0.1);
	expect(slots[1].components).toEqual([]);
});
