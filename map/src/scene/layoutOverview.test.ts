import { PerspectiveCamera, Vector3 } from "three";
import { describe, expect, it } from "vitest";
import { BLOCK_DIMENSIONS } from "@/block/dimensions";
import type { Block } from "@/graphql/generated";
import * as appearances from "@/theme/appearance";
import {
	getBlockHeight,
	LAYOUT_SCALE,
	layoutBlocks,
	type LayoutResult,
} from "./layout";
import {
	getLayoutOverview,
	layoutBlocksForView,
	type LayoutView,
	measureLayoutOverlap,
} from "./layoutOverview";

function block(
	name: string,
	connections: string[] = [],
	tags: string[] = [],
): Block {
	return {
		name,
		connections,
		tags,
		docs: [],
		frameworks: [],
		links: [],
		rawJsonString: "{}",
	};
}

const view: LayoutView = {
	...appearances.industrialAppearance.view,
	aspect: 16 / 9,
};
const fanout = [
	block("root", ["a", "b", "c", "d", "e"]),
	...["a", "b", "c", "d", "e"].map((name) => block(name)),
];
const fixtures = {
	chain: Array.from({ length: 6 }, (_, i) =>
		block(`chain-${i}`, i < 5 ? [`chain-${i + 1}`] : []),
	),
	fanout,
	dense: [
		block("a", ["d", "e", "f"]),
		block("b", ["d", "e", "f"]),
		block("c", ["d", "e", "f"]),
		block("d"),
		block("e"),
		block("f"),
	],
	disconnected: Array.from({ length: 8 }, (_, i) => block(`island-${i}`)),
	regions: [
		block("a", ["b", "c"], ["outer"]),
		block("b", ["d"], ["outer", "inner"]),
		block("c", ["d"], ["outer"]),
		block("d", [], ["inner"]),
	],
};

function expectAttachedRoutes(layout: LayoutResult) {
	for (const edge of layout.edges) {
		const source = layout.nodes.find((node) => node.id === edge.source);
		const target = layout.nodes.find((node) => node.id === edge.target);
		expect(source).toBeDefined();
		expect(target).toBeDefined();
		expect(edge.sections?.length).toBeGreaterThan(0);
		for (const section of edge.sections ?? []) {
			for (const [point, node] of [
				[section.startPoint, source],
				[section.endPoint, target],
			] as const) {
				if (!node) throw new Error("Missing edge endpoint");
				expect(point.x).toBeGreaterThanOrEqual(node.x - 1e-6);
				expect(point.x).toBeLessThanOrEqual(node.x + node.width + 1e-6);
				expect(point.y).toBeGreaterThanOrEqual(node.y - 1e-6);
				expect(point.y).toBeLessThanOrEqual(node.y + node.height + 1e-6);
				expect(
					Math.min(
						Math.abs(point.x - node.x),
						Math.abs(point.x - node.x - node.width),
						Math.abs(point.y - node.y),
						Math.abs(point.y - node.y - node.height),
					),
				).toBeLessThan(1e-6);
			}
			const points = [
				section.startPoint,
				...(section.bendPoints ?? []),
				section.endPoint,
			];
			for (let i = 1; i < points.length; i++) {
				expect(
					Math.min(
						Math.abs(points[i].x - points[i - 1].x),
						Math.abs(points[i].y - points[i - 1].y),
					),
				).toBeLessThan(1e-6);
			}
		}
	}
}

function expectVisibleInFrame(layout: LayoutResult, view: LayoutView) {
	const overview = getLayoutOverview(layout, view);
	// Reconstruct independently, as the rendered camera does.
	const camera = new PerspectiveCamera(
		view.fov,
		view.aspect,
		0.1,
		overview.camera.far,
	);
	camera.position.copy(overview.camera.position);
	camera.lookAt(overview.target);
	camera.updateMatrixWorld(true);
	for (const node of layout.nodes) {
		const height = getBlockHeight(node.id, layout);
		for (const x of [node.x, node.x + node.width]) {
			for (const z of [node.y, node.y + node.height]) {
				for (const y of [
					height - BLOCK_DIMENSIONS.height / 2,
					height + BLOCK_DIMENSIONS.height / 2 + 0.01,
				]) {
					const point = new Vector3(
						x * LAYOUT_SCALE,
						y,
						z * LAYOUT_SCALE,
					).project(camera);
					expect(Math.abs(point.x)).toBeLessThan(0.87);
					expect(Math.abs(point.y)).toBeLessThan(0.87);
					expect(point.z).toBeGreaterThan(-1);
					expect(point.z).toBeLessThan(1);
				}
			}
		}
	}
}

describe("camera-aware ELK layout", () => {
	it.each(Object.entries(fixtures))(
		"reduces overlap and preserves routes for %s",
		async (_name, blocks) => {
			const baseline = await layoutBlocks(blocks);
			const selected = await layoutBlocksForView(blocks, view);
			const before = measureLayoutOverlap(baseline, view);
			const after = measureLayoutOverlap(selected, view);
			expect(after).toBeLessThanOrEqual(before + 1e-6);
			if (before > 0.01) expect(after).toBeLessThan(before * 0.5);
			expectAttachedRoutes(selected);
			expectVisibleInFrame(selected, view);
			for (const group of selected.groups) {
				const members = selected.nodes.filter((node) =>
					group.nodeIds.includes(node.id),
				);
				expect(group.boundingBox).toEqual({
					minX: Math.min(...members.map((node) => node.x)),
					maxX: Math.max(...members.map((node) => node.x + node.width)),
					minY: Math.min(...members.map((node) => node.y)),
					maxY: Math.max(...members.map((node) => node.y + node.height)),
				});
			}
		},
		20000,
	);

	it.each(Object.entries(appearances))(
		"fits and improves %s in landscape and portrait",
		async (_name, appearance) => {
			const baseline = await layoutBlocks(fanout);
			for (const aspect of [16 / 9, 9 / 16]) {
				const themeView = { ...appearance.view, aspect };
				const selected = await layoutBlocksForView(fanout, themeView);
				expect(measureLayoutOverlap(selected, themeView)).toBeLessThanOrEqual(
					measureLayoutOverlap(baseline, themeView) + 1e-6,
				);
				expectVisibleInFrame(selected, themeView);
			}
		},
		20000,
	);

	it("compacts the previous coarse spacing without hiding cards", async () => {
		const previous = await layoutBlocks(fanout, {
			nodeSpacing: 120,
			layerSpacing: 240,
			componentSpacing: 120,
		});
		const compact = await layoutBlocksForView(fanout, view);
		expect(compact.width * compact.height).toBeLessThan(
			previous.width * previous.height * 0.95,
		);
		expect(measureLayoutOverlap(compact, view)).toBeLessThanOrEqual(
			measureLayoutOverlap(previous, view) + 1e-6,
		);
		expectAttachedRoutes(compact);
	});

	it("is deterministic and preserves compact layouts that are already clear", async () => {
		const first = await layoutBlocksForView(fanout, view);
		expect(await layoutBlocksForView(fanout, view)).toEqual(first);
		for (const blocks of [[], [block("alone")]]) {
			expect(await layoutBlocksForView(blocks, view)).toEqual(
				await layoutBlocks(blocks),
			);
		}
	});

	it("stops searching when the request becomes stale", async () => {
		const baseline = await layoutBlocks(fanout);
		expect(await layoutBlocksForView(fanout, view, () => false)).toEqual(
			baseline,
		);
	});
});
