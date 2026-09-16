import { Box3, MathUtils, PerspectiveCamera, Vector3 } from "three";
import { BLOCK_DIMENSIONS, BLOCK_LABEL_LIFT } from "@/block/dimensions";
import {
	calculateRegionCorners,
	getBlockHeight,
	LAYOUT_SCALE,
	type LayoutBlock,
	type LayoutOptions,
	type LayoutResult,
	layoutBlocks,
} from "./layout";

export interface LayoutView {
	position: [number, number, number];
	fov: number;
	aspect: number;
}

// Includes the normal docked card and side-slot tabs, but not transient slide-out
// animations. A small visual margin also protects the label on the top face.
const VISUAL_MARGIN = 0.08;
const FRAME_FILL = 0.86;
const SPACING_CANDIDATES = [
	[1, 1],
	[1, 1.6],
	[1.6, 1],
	[1.6, 1.6],
	[1, 2.5],
	[2.5, 1],
	[2.5, 2.5],
	[4, 4],
] as const;

function corners(bounds: Box3): Vector3[] {
	return [bounds.min.x, bounds.max.x].flatMap((x) =>
		[bounds.min.y, bounds.max.y].flatMap((y) =>
			[bounds.min.z, bounds.max.z].map((z) => new Vector3(x, y, z)),
		),
	);
}

export function getLayoutBlockBounds(layout: LayoutResult): Box3[] {
	return layout.nodes.map((node) => {
		const y = getBlockHeight(node.id, layout);
		return new Box3(
			new Vector3(
				node.x * LAYOUT_SCALE - VISUAL_MARGIN,
				y - BLOCK_DIMENSIONS.height / 2,
				node.y * LAYOUT_SCALE - VISUAL_MARGIN,
			),
			new Vector3(
				(node.x + node.width) * LAYOUT_SCALE + VISUAL_MARGIN,
				y + BLOCK_DIMENSIONS.height / 2 + BLOCK_LABEL_LIFT + VISUAL_MARGIN,
				(node.y + node.height) * LAYOUT_SCALE + VISUAL_MARGIN,
			),
		);
	});
}

/** The exact same fitted perspective camera is used for scoring and rendering. */
export function getLayoutOverview(layout: LayoutResult, view: LayoutView) {
	const blockBounds = getLayoutBlockBounds(layout);
	const bounds = new Box3();
	for (const block of blockBounds) bounds.union(block);
	for (const group of layout.groups) {
		if (group.nodeIds.length < 2) continue;
		const region = calculateRegionCorners(group, layout.groups);
		for (const point of Object.values(region)) {
			bounds.expandByPoint(new Vector3(...point));
		}
	}
	for (const edge of layout.edges) {
		for (const section of edge.sections ?? []) {
			for (const point of [
				section.startPoint,
				...(section.bendPoints ?? []),
				section.endPoint,
			]) {
				bounds.expandByPoint(
					new Vector3(point.x * LAYOUT_SCALE, 0, point.y * LAYOUT_SCALE),
				);
			}
		}
	}
	if (bounds.isEmpty())
		bounds.set(new Vector3(-1, 0, -1), new Vector3(1, 2, 1));

	const target = bounds.getCenter(new Vector3());
	const direction = new Vector3(...view.position).normalize();
	const camera = new PerspectiveCamera(
		view.fov,
		Math.max(0.1, view.aspect),
		0.1,
		1000,
	);
	camera.position.copy(target).add(direction);
	camera.lookAt(target);
	const inverseRotation = camera.quaternion.clone().invert();
	const tanVertical = Math.tan(MathUtils.degToRad(view.fov / 2)) * FRAME_FILL;
	const tanHorizontal = tanVertical * camera.aspect;
	let distance = new Vector3(...view.position).length();
	for (const corner of corners(bounds)) {
		const local = corner.sub(target).applyQuaternion(inverseRotation);
		distance = Math.max(
			distance,
			local.z + Math.abs(local.x) / tanHorizontal,
			local.z + Math.abs(local.y) / tanVertical,
		);
	}
	camera.position.copy(target).addScaledVector(direction, distance);
	camera.far = Math.max(
		1000,
		distance + bounds.getSize(new Vector3()).length() * 2,
	);
	camera.updateProjectionMatrix();
	camera.updateMatrixWorld(true);
	return { camera, target, blockBounds };
}

interface ScreenBounds {
	minX: number;
	maxX: number;
	minY: number;
	maxY: number;
}

function projectBounds(
	points: Vector3[],
	camera: PerspectiveCamera,
): ScreenBounds {
	const projected = points.map((point) => point.clone().project(camera));
	return {
		minX: Math.min(...projected.map((point) => point.x)),
		maxX: Math.max(...projected.map((point) => point.x)),
		minY: Math.min(...projected.map((point) => point.y)),
		maxY: Math.max(...projected.map((point) => point.y)),
	};
}

function area(bounds: ScreenBounds) {
	return (bounds.maxX - bounds.minX) * (bounds.maxY - bounds.minY);
}

function overlap(a: ScreenBounds, b: ScreenBounds) {
	return (
		Math.max(0, Math.min(a.maxX, b.maxX) - Math.max(a.minX, b.minX)) *
		Math.max(0, Math.min(a.maxY, b.maxY) - Math.max(a.minY, b.minY))
	);
}

/** Normalized overlap cannot improve merely by zooming out to shrink the map. */
export function measureLayoutOverlap(
	layout: LayoutResult,
	view: LayoutView,
): number {
	const { camera, blockBounds } = getLayoutOverview(layout, view);
	const projected = blockBounds.map((bounds) => {
		const points = corners(bounds);
		return {
			body: projectBounds(points, camera),
			label: projectBounds(
				points.filter((point) => point.y === bounds.max.y),
				camera,
			),
			depth: bounds
				.getCenter(new Vector3())
				.applyMatrix4(camera.matrixWorldInverse).z,
		};
	});
	let score = 0;
	for (let i = 0; i < projected.length; i++) {
		for (let j = i + 1; j < projected.length; j++) {
			const a = projected[i];
			const b = projected[j];
			const [front, back] = a.depth > b.depth ? [a, b] : [b, a];
			score +=
				overlap(a.body, b.body) /
				Math.max(1e-12, Math.min(area(a.body), area(b.body)));
			// Hiding a name is especially costly. Screen rectangles deliberately
			// overestimate silhouettes to leave comfortable visual clearance.
			score +=
				(2 * overlap(front.body, back.label)) /
				Math.max(1e-12, area(back.label));
		}
	}
	return score;
}

/** Bounded search; each candidate keeps ELK's own edge routing and tag bounds. */
export async function layoutBlocksForView(
	blocks: LayoutBlock[],
	view: LayoutView,
	isCurrent: () => boolean = () => true,
): Promise<LayoutResult> {
	let best = await layoutBlocks(blocks);
	let bestScore = measureLayoutOverlap(best, view);
	let spacing: LayoutOptions = { nodeSpacing: 120, layerSpacing: 150 };
	for (const [nodeFactor, layerFactor] of SPACING_CANDIDATES.slice(1)) {
		if (bestScore < 0.001 || !isCurrent()) break;
		const candidateSpacing = {
			nodeSpacing: 120 * nodeFactor,
			layerSpacing: 150 * layerFactor,
			componentSpacing: 120 * nodeFactor,
		};
		const candidate = await layoutBlocks(blocks, candidateSpacing);
		const score = measureLayoutOverlap(candidate, view);
		if (
			score < bestScore - 0.001 ||
			(Math.abs(score - bestScore) < 0.001 &&
				candidate.width * candidate.height < best.width * best.height)
		) {
			best = candidate;
			bestScore = score;
			spacing = candidateSpacing;
		}
	}
	// Trim the coarse search's excess clearance in two small steps. Each trial
	// gets fresh ELK routes and must preserve visibility while reducing area.
	// Try axes separately when reducing both would hide a card behind a neighbor.
	for (let round = 0; round < 2 && isCurrent() && blocks.length > 1; round++) {
		for (const [nodeFactor, layerFactor] of [
			[0.9, 0.9],
			[0.9, 1],
			[1, 0.9],
		]) {
			if (!isCurrent()) return best;
			const candidateSpacing = {
				nodeSpacing: (spacing.nodeSpacing ?? 120) * nodeFactor,
				layerSpacing: (spacing.layerSpacing ?? 150) * layerFactor,
				componentSpacing:
					spacing.componentSpacing === undefined
						? undefined
						: spacing.componentSpacing * nodeFactor,
			};
			const candidate = await layoutBlocks(blocks, candidateSpacing);
			const score = measureLayoutOverlap(candidate, view);
			if (
				score <= bestScore + 1e-6 &&
				candidate.width * candidate.height < best.width * best.height
			) {
				best = candidate;
				bestScore = score;
				spacing = candidateSpacing;
				break;
			}
		}
	}
	return best;
}
