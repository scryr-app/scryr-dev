import { Spherical, Vector3 } from "three";
import type { OrbitControls } from "three-stdlib";

interface FocusBlockOptions {
	/** Stable block identifier used to ignore the focused block in obstruction checks. */
	id?: string;
	/** World-space block center position. */
	position: [number, number, number];
	/** Block width in world units. */
	width: number;
	/** Block height in world units. */
	height: number;
	/** Block depth in world units. */
	depth: number;
}

interface RegisteredBlock {
	/** Stable block identifier. */
	id: string;
	/** World-space block center position. */
	position: [number, number, number];
	/** Block width in world units. */
	width: number;
	/** Block height in world units. */
	height: number;
	/** Block depth in world units. */
	depth: number;
}

interface SegmentRectIntersection {
	tMin: number;
	tMax: number;
}

const registeredBlocks = new Map<string, RegisteredBlock>();

function getSegmentRectIntersection(
	start: { x: number; z: number },
	end: { x: number; z: number },
	rect: { minX: number; maxX: number; minZ: number; maxZ: number },
): SegmentRectIntersection | null {
	const dx = end.x - start.x;
	const dz = end.z - start.z;
	let tMin = 0;
	let tMax = 1;

	const clip = (numerator: number, denominator: number) => {
		if (denominator === 0) {
			return numerator <= 0;
		}

		const t = numerator / denominator;
		if (denominator < 0) {
			if (t > tMax) return false;
			if (t > tMin) tMin = t;
			return true;
		}

		if (t < tMin) return false;
		if (t < tMax) tMax = t;
		return true;
	};

	const intersects =
		clip(rect.minX - start.x, dx) &&
		clip(start.x - rect.maxX, -dx) &&
		clip(rect.minZ - start.z, dz) &&
		clip(start.z - rect.maxZ, -dz);

	return intersects ? { tMin, tMax } : null;
}

function countObstructions({
	cameraPosition,
	targets,
	focusedBlockId,
}: {
	cameraPosition: Vector3;
	targets: Vector3[];
	focusedBlockId?: string;
}) {
	let obstructions = 0;

	for (const block of registeredBlocks.values()) {
		if (block.id === focusedBlockId) {
			continue;
		}

		const [x, y, z] = block.position;
		const horizontalMargin = 0.42;
		const verticalMargin = 0.12;
		const blockerBounds = {
			minX: x - block.width / 2 - horizontalMargin,
			maxX: x + block.width / 2 + horizontalMargin,
			minZ: z - block.depth / 2 - horizontalMargin,
			maxZ: z + block.depth / 2 + horizontalMargin,
		};
		const blockerBottom = y - block.height / 2 - verticalMargin;
		const blockerTop = y + block.height / 2 + verticalMargin;

		for (const target of targets) {
			const intersection = getSegmentRectIntersection(
				{ x: cameraPosition.x, z: cameraPosition.z },
				{ x: target.x, z: target.z },
				blockerBounds,
			);

			if (!intersection) {
				continue;
			}

			const yAtMin =
				cameraPosition.y + (target.y - cameraPosition.y) * intersection.tMin;
			const yAtMax =
				cameraPosition.y + (target.y - cameraPosition.y) * intersection.tMax;
			const rayMinY = Math.min(yAtMin, yAtMax);
			const rayMaxY = Math.max(yAtMin, yAtMax);
			const verticallyBlocked =
				rayMaxY >= blockerBottom && rayMinY <= blockerTop;

			if (verticallyBlocked) {
				obstructions += 1;
			}
		}
	}

	return obstructions;
}

function getFrontFaceTargets({
	x,
	y,
	z,
	width,
	height,
	depth,
}: {
	x: number;
	y: number;
	z: number;
	width: number;
	height: number;
	depth: number;
}) {
	const frontZ = z + depth / 2;
	const centerY = y + height * 0.08;
	const sideX = width * 0.44;
	const upperY = y + height * 0.3;

	return [
		new Vector3(x, centerY, frontZ),
		new Vector3(x - sideX, centerY, frontZ),
		new Vector3(x + sideX, centerY, frontZ),
		new Vector3(x - sideX, upperY, frontZ),
		new Vector3(x + sideX, upperY, frontZ),
	];
}

function getCandidateCameraPosition({
	angle,
	heightMultiplier,
	target,
	y,
	height,
	viewDistance,
}: {
	angle: number;
	heightMultiplier: number;
	target: Vector3;
	y: number;
	height: number;
	viewDistance: number;
}) {
	const cameraY = y + height * heightMultiplier;
	const verticalOffset = cameraY - target.y;
	const horizontalDistance = Math.sqrt(
		Math.max(1.25 ** 2, viewDistance ** 2 - verticalOffset ** 2),
	);

	return new Vector3(
		target.x + Math.sin(angle) * horizontalDistance,
		cameraY,
		target.z + Math.cos(angle) * horizontalDistance,
	);
}

/**
 * Module-level singleton so non-diagram components (e.g. MapTray)
 * can trigger camera actions without prop-drilling.
 */
export const cameraStore = {
	controls: null as OrbitControls | null,

	registerBlock(block: RegisteredBlock) {
		registeredBlocks.set(block.id, block);

		return () => {
			registeredBlocks.delete(block.id);
		};
	},

	focusBlock({ id, position, width, height, depth }: FocusBlockOptions) {
		const ctrl = this.controls;
		if (!ctrl) return;

		const [x, y, z] = position;
		const target = new Vector3(x, y + height * 0.08, z + depth / 2);
		const focusTargets = getFrontFaceTargets({ x, y, z, width, height, depth });
		const viewDistance = Math.max(2.55, width * 0.88, height * 1.28);
		const candidateAngles = [
			0,
			Math.PI / 8,
			-Math.PI / 8,
			Math.PI / 5,
			-Math.PI / 5,
			Math.PI / 3.2,
			-Math.PI / 3.2,
			Math.PI / 2.45,
			-Math.PI / 2.45,
		];
		const [bestCandidate] = candidateAngles
			.flatMap((angle) =>
				[0.36, 0.64, 0.94].map((heightMultiplier) => {
					const cameraPosition = getCandidateCameraPosition({
						angle,
						heightMultiplier,
						target,
						y,
						height,
						viewDistance,
					});
					const obstructionCount = countObstructions({
						cameraPosition,
						targets: focusTargets,
						focusedBlockId: id,
					});

					return {
						cameraPosition,
						score:
							obstructionCount * 1000 + Math.abs(angle) * 10 + heightMultiplier,
					};
				}),
			)
			.sort((a, b) => a.score - b.score);

		ctrl.target.copy(target);
		ctrl.object.position.copy(bestCandidate?.cameraPosition ?? target);
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
