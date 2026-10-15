import { Box3, PerspectiveCamera, Ray, Vector3 } from "three";
import { describe, expect, it } from "vitest";
import * as appearances from "@/theme/appearance";
import { type FocusBlock, getBlockFocus } from "./blockFocus";
import { DEFAULT_FOCUS_VIEWPORT, type FocusViewport } from "./focusViewport";

const block: FocusBlock = {
	id: "selected",
	position: [4, 1.2, -3],
	width: 3,
	height: 2,
	depth: 1,
};

function focusedCamera(
	camera: PerspectiveCamera,
	blockers: FocusBlock[] = [],
	viewport = DEFAULT_FOCUS_VIEWPORT,
) {
	const focus = getBlockFocus(block, camera, [block, ...blockers], {
		viewport,
	});
	camera.position.copy(focus.position);
	camera.lookAt(focus.target);
	camera.updateMatrixWorld(true);
	return focus;
}

function expectFullCard(camera: PerspectiveCamera, viewport: FocusViewport) {
	const [x, y, z] = block.position;
	for (const dx of [-block.width / 2, block.width / 2]) {
		for (const dy of [-block.height / 2, block.height / 2]) {
			const point = new Vector3(
				x + dx,
				y + dy,
				z + block.depth / 2 + 0.076,
			).project(camera);
			expect(point.x).toBeGreaterThanOrEqual(viewport.minX);
			expect(point.x).toBeLessThanOrEqual(viewport.maxX);
			expect(point.y).toBeGreaterThanOrEqual(viewport.minY);
			expect(point.y).toBeLessThanOrEqual(viewport.maxY);
			expect(point.z).toBeGreaterThan(-1);
			expect(point.z).toBeLessThan(1);
		}
	}
}

function expectUnobstructed(camera: PerspectiveCamera, blockers: FocusBlock[]) {
	const [x, y, z] = block.position;
	// Independent dense segment sampling across the actual card surface.
	for (let ix = 0; ix <= 10; ix++) {
		for (let iy = 0; iy <= 10; iy++) {
			const target = new Vector3(
				x + (ix / 10 - 0.5) * 2.8,
				y + (iy / 10 - 0.5) * 1.8,
				z + 0.576,
			);
			const ray = new Ray(
				camera.position,
				target.clone().sub(camera.position).normalize(),
			);
			for (const blocker of blockers) {
				const center = new Vector3(...blocker.position);
				const half = new Vector3(
					blocker.width / 2,
					blocker.height / 2,
					blocker.depth / 2,
				);
				const hit = ray.intersectBox(
					new Box3(center.clone().sub(half), center.clone().add(half)),
					new Vector3(),
				);
				if (hit)
					expect(hit.distanceTo(camera.position)).toBeGreaterThan(
						target.distanceTo(camera.position),
					);
			}
		}
	}
}

describe("individual block focus", () => {
	it.each(Object.entries(appearances))(
		"frames the full card for %s",
		(_name, appearance) => {
			for (const aspect of [16 / 9, 9 / 16, 0.35]) {
				for (const zoom of [1, 2]) {
					const camera = new PerspectiveCamera(
						appearance.view.fov,
						aspect,
						0.1,
						1000,
					);
					camera.zoom = zoom;
					camera.updateProjectionMatrix();
					const focus = focusedCamera(camera);
					expect(focus.obstructionCount).toBe(0);
					expectFullCard(camera, DEFAULT_FOCUS_VIEWPORT);
					const offset = camera.position.clone().sub(focus.target);
					expect(Math.acos(offset.y / offset.length())).toBeLessThanOrEqual(
						Math.PI / 2 - Math.PI / 18,
					);
				}
			}
		},
	);

	it.each([
		{ id: "front", position: [4, 1.2, 0], width: 3, height: 2, depth: 1 },
		{
			id: "lower-corner",
			position: [2.9, 0.55, -1],
			width: 0.45,
			height: 0.5,
			depth: 0.4,
		},
		{ id: "raised", position: [4, 2.5, 0], width: 2, height: 1.2, depth: 1 },
	] satisfies FocusBlock[])("avoids $id across the whole card", (blocker) => {
		const camera = new PerspectiveCamera(40, 16 / 9, 0.1, 1000);
		const focus = focusedCamera(camera, [blocker]);
		expect(focus.obstructionCount).toBe(0);
		expectFullCard(camera, DEFAULT_FOCUS_VIEWPORT);
		expectUnobstructed(camera, [blocker]);
	});

	it("ignores the selected block and blocks behind its card", () => {
		const camera = new PerspectiveCamera(32, 0.6, 0.1, 1000);
		const baseline = focusedCamera(camera);
		const behind: FocusBlock = {
			id: "behind",
			position: [4, 1.2, -6],
			width: 6,
			height: 4,
			depth: 1,
		};
		const focus = focusedCamera(camera, [behind]);
		expect(focus).toEqual(baseline);
	});

	it("keeps the card in free canvas space beside the editor and above the toolbar", () => {
		const viewport = { minX: -0.05, maxX: 0.9, minY: -0.72, maxY: 0.9 };
		const camera = new PerspectiveCamera(35, 1.4, 0.1, 1000);
		focusedCamera(camera, [], viewport);
		expectFullCard(camera, viewport);
	});

	it.each([
		{ minX: 0.45, maxX: 0.95, minY: -0.9, maxY: 0.9 },
		{ minX: -0.95, maxX: -0.45, minY: -0.9, maxY: 0.9 },
		{ minX: -0.9, maxX: 0.9, minY: -0.95, maxY: -0.45 },
	])("keeps an off-center focus in front of the card", (viewport) => {
		const camera = new PerspectiveCamera(65, 2.5, 0.1, 1000);
		const blocker: FocusBlock = {
			id: "front",
			position: [4, 1.2, 0],
			width: 3,
			height: 2,
			depth: 1,
		};
		const focus = focusedCamera(camera, [blocker], viewport);
		expect(camera.position.z).toBeGreaterThan(
			block.position[2] + block.depth / 2 + 0.1,
		);
		expect(focus.obstructionCount).toBe(0);
		expectFullCard(camera, viewport);
		expectUnobstructed(camera, [blocker]);
	});

	it("respects a steeper camera polar limit while fitting", () => {
		const camera = new PerspectiveCamera(32, 0.4, 0.1, 1000);
		const focus = getBlockFocus(block, camera, [], {
			maxPolarAngle: Math.PI / 4,
		});
		camera.position.copy(focus.position);
		camera.lookAt(focus.target);
		camera.updateMatrixWorld(true);
		expectFullCard(camera, DEFAULT_FOCUS_VIEWPORT);
		const direction = focus.position.clone().sub(focus.target).normalize();
		expect(Math.acos(direction.y)).toBeLessThanOrEqual(Math.PI / 4 + 1e-8);
	});
});
