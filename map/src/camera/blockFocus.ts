import {
	Box3,
	Frustum,
	MathUtils,
	type PerspectiveCamera,
	Plane,
	Vector3,
} from "three";
import { DEFAULT_FOCUS_VIEWPORT, type FocusViewport } from "./focusViewport";

export interface FocusBlock {
	id?: string;
	position: [number, number, number];
	width: number;
	height: number;
	depth: number;
}

const FACE_CLEARANCE = 0.08; // Includes the docked card's surface and text.

function blockBounds(block: FocusBlock, margin = 0) {
	const center = new Vector3(...block.position);
	const half = new Vector3(
		block.width / 2 + margin,
		block.height / 2 + margin,
		block.depth / 2 + margin,
	);
	return new Box3(center.clone().sub(half), center.clone().add(half));
}

function boxCorners(box: Box3) {
	return [box.min.x, box.max.x].flatMap((x) =>
		[box.min.y, box.max.y].flatMap((y) =>
			[box.min.z, box.max.z].map((z) => new Vector3(x, y, z)),
		),
	);
}

/** The volume between the camera and the entire card, including its lower edge. */
function cardSightVolume(position: Vector3, face: Vector3[]) {
	const center = face
		.reduce((sum, point) => sum.add(point), new Vector3())
		.multiplyScalar(1 / face.length);
	const inside = center.clone().lerp(position, 0.5);
	const planes = face.map((point, i) => {
		const plane = new Plane().setFromCoplanarPoints(
			position,
			point,
			face[(i + 1) % face.length],
		);
		if (plane.distanceToPoint(inside) < 0) plane.negate();
		return plane;
	});
	planes.push(new Plane(new Vector3(0, 0, 1), -center.z));
	planes.push(
		new Plane().setFromNormalAndCoplanarPoint(
			center.clone().sub(position).normalize(),
			position,
		),
	);
	return new Frustum(...planes);
}

/** Fit the block/card using the real perspective projection, then avoid blockers. */
export function getBlockFocus(
	block: FocusBlock,
	camera: PerspectiveCamera,
	neighbors: Iterable<FocusBlock>,
	{
		minPolarAngle = 0,
		maxPolarAngle = Math.PI / 2 - Math.PI / 18,
		viewport = DEFAULT_FOCUS_VIEWPORT,
	}: {
		minPolarAngle?: number;
		maxPolarAngle?: number;
		viewport?: FocusViewport;
	} = {},
) {
	const bounds = blockBounds(block, FACE_CLEARANCE);
	const points = boxCorners(bounds);
	const [x, y] = block.position;
	const center = new Vector3(x, y, bounds.max.z);
	const face = [
		new Vector3(bounds.min.x, bounds.min.y, bounds.max.z),
		new Vector3(bounds.max.x, bounds.min.y, bounds.max.z),
		new Vector3(bounds.max.x, bounds.max.y, bounds.max.z),
		new Vector3(bounds.min.x, bounds.max.y, bounds.max.z),
	];
	const blockers = Array.from(neighbors)
		.filter((other) => other.id !== block.id)
		.map((other) => blockBounds(other, 0.04));
	const tanY = Math.tan(MathUtils.degToRad(camera.getEffectiveFOV() / 2));
	const tanX = tanY * camera.aspect;
	const centerX = (viewport.minX + viewport.maxX) / 2;
	const centerY = (viewport.minY + viewport.maxY) / 2;
	const candidateCamera = camera.clone();
	const candidates = [];
	for (const angle of [
		0,
		Math.PI / 8,
		-Math.PI / 8,
		Math.PI / 5,
		-Math.PI / 5,
		Math.PI / 3,
		-Math.PI / 3,
	]) {
		for (const elevation of [12, 22, 35, 50, 65]) {
			const polar = MathUtils.clamp(
				MathUtils.degToRad(90 - elevation),
				minPolarAngle,
				maxPolarAngle,
			);
			const direction = new Vector3(
				Math.sin(polar) * Math.sin(angle),
				Math.cos(polar),
				Math.sin(polar) * Math.cos(angle),
			);
			candidateCamera.position.copy(center).add(direction);
			candidateCamera.lookAt(center);
			const inverseRotation = candidateCamera.quaternion.clone().invert();
			let distance = camera.near + FACE_CLEARANCE;
			for (const point of points) {
				const local = point
					.clone()
					.sub(center)
					.applyQuaternion(inverseRotation);
				// Off-center framing keeps the complete card clear of floating UI.
				distance = Math.max(
					distance,
					(local.x / tanX + viewport.maxX * local.z) /
						(viewport.maxX - centerX),
					(-local.x / tanX - viewport.minX * local.z) /
						(centerX - viewport.minX),
					(local.y / tanY + viewport.maxY * local.z) /
						(viewport.maxY - centerY),
					(-local.y / tanY - viewport.minY * local.z) /
						(centerY - viewport.minY),
					local.z + camera.near + FACE_CLEARANCE,
				);
			}
			const offset = new Vector3(
				-centerX * distance * tanX,
				-centerY * distance * tanY,
				0,
			).applyQuaternion(candidateCamera.quaternion);
			const target = center.clone().add(offset);
			const position = target.clone().addScaledVector(direction, distance);
			// Off-center framing can move an angled camera behind the front face.
			// Such a view would look through the selected block itself.
			if (position.z <= bounds.max.z + camera.near) continue;
			const sight = cardSightVolume(position, face);
			const obstructionCount = blockers.filter((blocker) =>
				sight.intersectsBox(blocker),
			).length;
			candidates.push({
				position,
				target,
				obstructionCount,
				preference: Math.abs(angle) + MathUtils.degToRad(elevation),
			});
		}
	}
	candidates.sort(
		(a, b) =>
			a.obstructionCount - b.obstructionCount || a.preference - b.preference,
	);
	return candidates[0];
}
