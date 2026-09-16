import { PerspectiveCamera, Spherical, Vector3 } from "three";
import type { OrbitControls } from "three-stdlib";
import { currentTheme } from "@/theme/theme";

import { type FocusBlock, getBlockFocus } from "./blockFocus";
import { getFocusViewport } from "./focusViewport";

const registeredBlocks = new Map<string, FocusBlock>();

/**
 * Module-level singleton so non-diagram components (e.g. MapTray)
 * can trigger camera actions without prop-drilling.
 */
export const cameraStore = {
	controls: null as OrbitControls | null,

	registerBlock(block: FocusBlock & { id: string }) {
		registeredBlocks.set(block.id, block);

		return () => {
			registeredBlocks.delete(block.id);
		};
	},

	focusBlock(block: FocusBlock) {
		const ctrl = this.controls;
		if (!ctrl || !(ctrl.object instanceof PerspectiveCamera)) return;
		// Finish pending orbit/pan inertia before applying the fitted pose.
		const damping = ctrl.enableDamping;
		ctrl.enableDamping = false;
		ctrl.update();
		ctrl.enableDamping = damping;
		const focus = getBlockFocus(block, ctrl.object, registeredBlocks.values(), {
			minPolarAngle: ctrl.minPolarAngle,
			maxPolarAngle: ctrl.maxPolarAngle,
			viewport: getFocusViewport(ctrl.domElement),
		});
		ctrl.target.copy(focus.target);
		ctrl.object.position.copy(focus.position);
		ctrl.update();
	},

	setTopDownView() {
		const ctrl = this.controls;
		if (!ctrl) return;
		const distance = ctrl.object.position.distanceTo(ctrl.target);
		ctrl.object.position.set(
			ctrl.target.x,
			ctrl.target.y + distance,
			ctrl.target.z,
		);
		ctrl.update();
	},

	setIsometricView() {
		const ctrl = this.controls;
		if (!ctrl) return;
		const distance = ctrl.object.position.distanceTo(ctrl.target);
		const direction = new Vector3(
			...currentTheme.appearance.view.position,
		).normalize();
		ctrl.object.position.copy(ctrl.target).addScaledVector(direction, distance);
		ctrl.update();
	},

	zoomIn(factor = 0.97) {
		const ctrl = this.controls;
		if (!ctrl) return;
		const offset = new Vector3().subVectors(ctrl.object.position, ctrl.target);
		offset.multiplyScalar(factor);
		ctrl.object.position.copy(ctrl.target).add(offset);
		ctrl.update();
	},

	zoomOut(factor = 0.97) {
		const ctrl = this.controls;
		if (!ctrl) return;
		const offset = new Vector3().subVectors(ctrl.object.position, ctrl.target);
		offset.multiplyScalar(1 / factor);
		ctrl.object.position.copy(ctrl.target).add(offset);
		ctrl.update();
	},

	panUp(distance = 0.08) {
		if (this.controls) {
			this.controls.target.y += distance;
			this.controls.object.position.y += distance;
			this.controls.update();
		}
	},

	panDown(distance = 0.08) {
		if (this.controls) {
			this.controls.target.y -= distance;
			this.controls.object.position.y -= distance;
			this.controls.update();
		}
	},

	panLeft(distance = 0.08) {
		if (this.controls) {
			this.controls.target.x -= distance;
			this.controls.object.position.x -= distance;
			this.controls.update();
		}
	},

	panRight(distance = 0.08) {
		if (this.controls) {
			this.controls.target.x += distance;
			this.controls.object.position.x += distance;
			this.controls.update();
		}
	},

	rotateLeft(angle = 0.02) {
		const ctrl = this.controls;
		if (!ctrl) return;
		const offset = new Vector3().subVectors(ctrl.object.position, ctrl.target);
		const spherical = new Spherical().setFromVector3(offset);
		spherical.theta -= angle;
		spherical.makeSafe();
		offset.setFromSpherical(spherical);
		ctrl.object.position.copy(ctrl.target).add(offset);
		ctrl.update();
	},

	rotateRight(angle = 0.02) {
		const ctrl = this.controls;
		if (!ctrl) return;
		const offset = new Vector3().subVectors(ctrl.object.position, ctrl.target);
		const spherical = new Spherical().setFromVector3(offset);
		spherical.theta += angle;
		spherical.makeSafe();
		offset.setFromSpherical(spherical);
		ctrl.object.position.copy(ctrl.target).add(offset);
		ctrl.update();
	},

	rotateUp(angle = 0.02) {
		const ctrl = this.controls;
		if (!ctrl) return;
		const offset = new Vector3().subVectors(ctrl.object.position, ctrl.target);
		const spherical = new Spherical().setFromVector3(offset);
		spherical.phi = Math.max(
			ctrl.minPolarAngle,
			Math.min(ctrl.maxPolarAngle, spherical.phi - angle),
		);
		spherical.makeSafe();
		offset.setFromSpherical(spherical);
		ctrl.object.position.copy(ctrl.target).add(offset);
		ctrl.update();
	},

	rotateDown(angle = 0.02) {
		const ctrl = this.controls;
		if (!ctrl) return;
		const offset = new Vector3().subVectors(ctrl.object.position, ctrl.target);
		const spherical = new Spherical().setFromVector3(offset);
		spherical.phi = Math.max(
			ctrl.minPolarAngle,
			Math.min(ctrl.maxPolarAngle, spherical.phi + angle),
		);
		spherical.makeSafe();
		offset.setFromSpherical(spherical);
		ctrl.object.position.copy(ctrl.target).add(offset);
		ctrl.update();
	},
};
