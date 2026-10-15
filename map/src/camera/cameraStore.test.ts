// @vitest-environment jsdom
import { PerspectiveCamera, Vector3 } from "three";
import { OrbitControls } from "three-stdlib";
import { afterEach, expect, it, vi } from "vitest";
import { cameraStore } from "./cameraStore";

const cleanups: Array<() => void> = [];
afterEach(() => {
	for (const cleanup of cleanups.splice(0)) cleanup();
	cameraStore.controls?.dispose();
	cameraStore.controls = null;
	document.body.replaceChildren();
});

it.each([16 / 9, 9 / 16])(
	"frames the full card after OrbitControls.update at aspect %s",
	(aspect) => {
		const canvas = document.createElement("canvas");
		document.body.append(canvas);
		vi.spyOn(canvas, "getBoundingClientRect").mockReturnValue(
			new DOMRect(0, 0, aspect * 1000, 1000),
		);
		const camera = new PerspectiveCamera(32, aspect, 0.1, 1000);
		camera.position.set(8, 6, 8);
		const controls = new OrbitControls(camera, canvas);
		controls.maxPolarAngle = Math.PI / 2 - Math.PI / 18;
		cameraStore.controls = controls;
		const block = {
			id: "focus",
			position: [0, 1.2, 0] as [number, number, number],
			width: 3,
			height: 2,
			depth: 1,
		};
		cleanups.push(cameraStore.registerBlock(block));
		cameraStore.focusBlock(block);
		controls.update();
		camera.updateMatrixWorld(true);
		for (const x of [-1.4, 1.4]) {
			for (const y of [0.3, 2.1]) {
				const point = new Vector3(x, y, 0.576).project(camera);
				expect(Math.abs(point.x)).toBeLessThan(0.82);
				expect(Math.abs(point.y)).toBeLessThan(0.82);
			}
		}
	},
);

it("clears pending orbit inertia before focusing", () => {
	const camera = new PerspectiveCamera(40, 1.4, 0.1, 1000);
	camera.position.set(8, 6, 8);
	const controls = new OrbitControls(camera, document.createElement("canvas"));
	controls.enableDamping = true;
	controls.setAzimuthalAngle(1.8);
	controls.setPolarAngle(0.9);
	cameraStore.controls = controls;
	cameraStore.focusBlock({
		id: "focus",
		position: [0, 1.2, 0],
		width: 3,
		height: 2,
		depth: 1,
	});
	const position = camera.position.clone();
	const target = controls.target.clone();
	for (let i = 0; i < 20; i++) controls.update();
	expect(camera.position.distanceTo(position)).toBeLessThan(1e-8);
	expect(controls.target.distanceTo(target)).toBeLessThan(1e-8);
	expect(controls.enableDamping).toBe(true);
});
