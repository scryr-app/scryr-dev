import * as ELK from "elkjs";
import type { Block } from "@/graphql/generated.ts";

const elk = new ELK.default();

interface ElkLayoutNode {
	id: string;
	x?: number;
	y?: number;
	width?: number;
	height?: number;
}

interface ElkLayoutSection {
	startPoint: { x: number; y: number };
	endPoint: { x: number; y: number };
	bendPoints?: Array<{ x: number; y: number }>;
}

interface ElkLayoutEdge {
	id: string;
	sources?: string[];
	targets?: string[];
	sections?: ElkLayoutSection[];
}

interface ElkLayoutGraph {
	children?: ElkLayoutNode[];
	edges?: ElkLayoutEdge[];
	width?: number;
	height?: number;
}

export interface LayoutNode {
	id: string;
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface LayoutEdge {
	id: string;
	source: string;
	target: string;
	sections?: Array<{
		startPoint: { x: number; y: number };
		endPoint: { x: number; y: number };
		bendPoints?: Array<{ x: number; y: number }>;
	}>;
}

export interface LayoutGroup {
	id: string;
	tag: string;
	nodeIds: string[];
	boundingBox: {
		minX: number;
		minY: number;
		maxX: number;
		maxY: number;
	};
	/**
	 * ELK-coordinate position for the region label sign.
	 * Placed at the front edge of the region facing the camera
	 * (camera is at world [13,12,13] → max ELK-Y = max world-Z edge).
	 */
	signPosition: { x: number; y: number };
}

export interface LayoutResult {
	nodes: LayoutNode[];
	edges: LayoutEdge[];
	groups: LayoutGroup[];
	width: number;
	height: number;
}

export interface LayoutOptions {
	algorithm?: "layered" | "force" | "stress" | "mrtree" | "radial";
	direction?: "RIGHT" | "LEFT" | "DOWN" | "UP";
	nodeSpacing?: number;
	layerSpacing?: number;
	nodeWidth?: number;
	nodeHeight?: number;
}

function normalizeGraphId(value: string | null | undefined): string | null {
	if (typeof value !== "string") {
		return null;
	}

	const normalized = value.trim();
	return normalized.length > 0 ? normalized : null;
}

/**
 * Layout blocks using ELK (Eclipse Layout Kernel)
 * Automatically positions nodes and routes edges based on connections
 * Maintains flat layout with tag information in node metadata
 */
export async function layoutBlocks(
	blocks: Block[],
	options: LayoutOptions = {},
): Promise<LayoutResult> {
	const {
		algorithm = "layered",
		direction = "DOWN",
		nodeSpacing = 120,
		layerSpacing = 150,
		// Node dimensions match actual BlockOpenRight size (3×1 in world coords)
		// Scaled by 50 for ELK coordinate system (1/50 scale factor)
		nodeWidth = 150, // 3 units × 50 = 150
		nodeHeight = 50, // 1 unit × 50 = 50
	} = options;

	const layoutBlocks = blocks
		.map((block) => {
			const name = normalizeGraphId(block.name);
			if (!name) {
				return null;
			}

			return {
				...block,
				name,
				connections: block.connections
					.map((connection) => normalizeGraphId(connection))
					.filter((connection): connection is string => connection !== null),
				tags: block.tags
					.map((tag) => normalizeGraphId(tag))
					.filter((tag): tag is string => tag !== null),
			};
		})
		.filter((block) => block !== null);

	const nodeIds = new Set(layoutBlocks.map((block) => block.name));
	const invalidConnections: string[] = [];

	// Build flat ELK graph (no nesting to avoid overlaps)
	// Blocks are positioned based on connections, not grouping
	const graph = {
		id: "root",
		layoutOptions: {
			"elk.algorithm": algorithm,
			"elk.direction": direction,
			"elk.spacing.nodeNode": nodeSpacing.toString(),
			"elk.layered.spacing.nodeNodeBetweenLayers": layerSpacing.toString(),
			"elk.edgeRouting": "ORTHOGONAL",
			"elk.spacing.edgeNode": "50",
			"elk.spacing.edgeEdge": "30",
			"elk.aspectRatio": "1.0", // Aim for square-like layout (1:1 ratio)
			"elk.layered.nodePlacement.strategy": "NETWORK_SIMPLEX", // Better distribution
		},
		children: layoutBlocks.map((block) => ({
			id: block.name,
			width: nodeWidth,
			height: nodeHeight,
			labels: [{ text: block.name }],
			properties: {
				tags: block.tags.join(","),
			},
		})),
		edges: layoutBlocks.flatMap((block) =>
			block.connections.flatMap((target) => {
				if (!nodeIds.has(target)) {
					invalidConnections.push(`${block.name} -> ${target}`);
					return [];
				}

				return [
					{
						id: `${block.name}-${target}`,
						sources: [block.name],
						targets: [target],
					},
				];
			}),
		),
	};

	if (invalidConnections.length > 0) {
		console.warn(
			`Skipping ${invalidConnections.length} invalid graph connection(s):`,
			invalidConnections,
		);
	}

	// Run ELK layout
	const layouted = (await elk.layout(graph)) as ElkLayoutGraph;

	// Extract positioned nodes
	const nodes: LayoutNode[] =
		layouted.children?.map((node) => ({
			id: node.id,
			x: node.x ?? 0,
			y: node.y ?? 0,
			width: node.width ?? nodeWidth,
			height: node.height ?? nodeHeight,
		})) ?? [];

	// Calculate center of all nodes to center layout around (0,0)
	if (nodes.length > 0) {
		const minX = Math.min(...nodes.map((n) => n.x));
		const maxX = Math.max(...nodes.map((n) => n.x + n.width));
		const minY = Math.min(...nodes.map((n) => n.y));
		const maxY = Math.max(...nodes.map((n) => n.y + n.height));

		const centerX = (minX + maxX) / 2;
		const centerY = (minY + maxY) / 2;

		// Offset all nodes to center them at (0,0)
		nodes.forEach((node) => {
			node.x -= centerX;
			node.y -= centerY;
		});
	}

	// Extract routed edges
	const edges: LayoutEdge[] =
		layouted.edges?.map((edge) => ({
			id: edge.id,
			source: edge.sources?.[0] ?? "",
			target: edge.targets?.[0] ?? "",
			sections: edge.sections?.map((section) => ({
				startPoint: section.startPoint,
				endPoint: section.endPoint,
				bendPoints: section.bendPoints,
			})),
		})) ?? [];

	// Offset edge points to match node centering
	if (nodes.length > 0) {
		const allNodePositions =
			layouted.children?.map((node) => ({
				x: node.x ?? 0,
				y: node.y ?? 0,
			})) ?? [];

		const minX = Math.min(...allNodePositions.map((n) => n.x));
		const maxX = Math.max(
			...allNodePositions.map(
				(n) => n.x + (layouted.children?.[0]?.width ?? nodeWidth),
			),
		);
		const minY = Math.min(...allNodePositions.map((n) => n.y));
		const maxY = Math.max(
			...allNodePositions.map(
				(n) => n.y + (layouted.children?.[0]?.height ?? nodeHeight),
			),
		);

		const centerX = (minX + maxX) / 2;
		const centerY = (minY + maxY) / 2;

		edges.forEach((edge) => {
			if (edge.sections) {
				edge.sections.forEach((section) => {
					section.startPoint.x -= centerX;
					section.startPoint.y -= centerY;
					section.endPoint.x -= centerX;
					section.endPoint.y -= centerY;
					if (section.bendPoints) {
						section.bendPoints.forEach((bendPoint) => {
							bendPoint.x -= centerX;
							bendPoint.y -= centerY;
						});
					}
				});
			}
		});
	}

	// Calculate tag-based groups with bounding boxes
	const tagGroups = new Map<string, string[]>();
	layoutBlocks.forEach((block) => {
		const { name } = block;
		block.tags.forEach((tag) => {
			if (!tagGroups.has(tag)) {
				tagGroups.set(tag, []);
			}
			tagGroups.get(tag)?.push(name);
		});
	});

	const groups: LayoutGroup[] = Array.from(tagGroups.entries())
		.map(([tag, nodeIds]) => {
			// Find bounding box for all nodes in this group
			const groupNodes = nodes.filter((n) => nodeIds.includes(n.id));
			if (groupNodes.length === 0) {
				return null;
			}
			const minX = Math.min(...groupNodes.map((n) => n.x));
			const maxX = Math.max(...groupNodes.map((n) => n.x + n.width));
			const minY = Math.min(...groupNodes.map((n) => n.y));
			const maxY = Math.max(...groupNodes.map((n) => n.y + n.height));

			// Sign sits at the front edge of the region facing the camera.
			// Camera is at world [13,12,13] → positive world-Z = maximum ELK-Y.
			// Place horizontally centred on the region, at the max-Y (front) edge.
			const signPosition = {
				x: (minX + maxX) / 2,
				y: maxY,
			};

			return {
				id: `tag-${tag}`,
				tag,
				nodeIds,
				boundingBox: { minX, maxX, minY, maxY },
				signPosition,
			};
		})
		.filter((g) => g !== null) as LayoutGroup[];

	return {
		nodes,
		edges,
		groups,
		width: layouted.width ?? 0,
		height: layouted.height ?? 0,
	};
}

/**
 * Convert layout positions to 3D world coordinates
 * Scales down the layout to fit in the 3D scene
 */
export function toWorldCoordinates(
	x: number,
	y: number,
	scale = 1 / 50,
	height: number = 0,
): [number, number, number] {
	return [x * scale, 0 + height, y * scale];
}

/**
 * Get node position in world coordinates
 */
export function getNodeWorldPosition(
	nodeId: string,
	layout: LayoutResult,
	scale = 1 / 50,
): [number, number, number] | null {
	const node = layout.nodes.find((n) => n.id === nodeId);
	if (!node) return null;

	// Use center of node
	return toWorldCoordinates(
		node.x + node.width / 2,
		node.y + node.height / 2,
		scale,
	);
}

/**
 * Get edge path points in world coordinates
 */
export function getEdgeWorldPath(
	edgeId: string,
	layout: LayoutResult,
	scale = 1 / 50,
): Array<[number, number, number]> | null {
	const edge = layout.edges.find((e) => e.id === edgeId);
	if (!edge?.sections || edge.sections.length === 0) return null;

	const section = edge.sections[0];
	const points: Array<[number, number, number]> = [];

	// Start point
	points.push(
		toWorldCoordinates(section.startPoint.x, section.startPoint.y, scale),
	);

	// Bend points
	if (section.bendPoints) {
		for (const bendPoint of section.bendPoints) {
			points.push(toWorldCoordinates(bendPoint.x, bendPoint.y, scale));
		}
	}

	// End point
	points.push(
		toWorldCoordinates(section.endPoint.x, section.endPoint.y, scale),
	);

	return points;
}

/**
 * Calculate the area of a group including its margin
 */
function calculateGroupArea(group: LayoutGroup, marginPercent = 0.05): number {
	const { minX, maxX, minY, maxY } = group.boundingBox;
	const width = maxX - minX;
	const height = maxY - minY;
	const marginX = width * marginPercent;
	const marginY = height * marginPercent;
	return (width + marginX * 2) * (height + marginY * 2);
}

/**
 * Calculate expanded bounds for a group including margin
 */
function calculateGroupBounds(
	group: LayoutGroup,
	marginPercent = 0.05,
): { minX: number; maxX: number; minY: number; maxY: number } {
	const { minX, maxX, minY, maxY } = group.boundingBox;
	const width = maxX - minX;
	const height = maxY - minY;
	const marginX = width * marginPercent;
	const marginY = height * marginPercent;

	return {
		minX: minX - marginX,
		maxX: maxX + marginX,
		minY: minY - marginY,
		maxY: maxY + marginY,
	};
}

/**
 * Check if two bounding boxes overlap
 */
function boundsOverlap(
	bounds1: { minX: number; maxX: number; minY: number; maxY: number },
	bounds2: { minX: number; maxX: number; minY: number; maxY: number },
): boolean {
	return !(
		bounds1.maxX < bounds2.minX ||
		bounds2.maxX < bounds1.minX ||
		bounds1.maxY < bounds2.minY ||
		bounds2.maxY < bounds1.minY
	);
}

/**
 * Calculate z-offset for a group based on overlaps with larger groups
 * Smaller groups that overlap with larger groups are lifted higher
 */
export function calculateGroupZOffset(
	group: LayoutGroup,
	allGroups: LayoutGroup[],
): number {
	const thisArea = calculateGroupArea(group);
	const thisBounds = calculateGroupBounds(group);
	let largerOverlapCount = 0;

	allGroups.forEach((otherGroup) => {
		if (group.id === otherGroup.id) return;

		const otherArea = calculateGroupArea(otherGroup);
		const otherBounds = calculateGroupBounds(otherGroup);

		// If this group overlaps with a larger group, increment counter
		if (boundsOverlap(thisBounds, otherBounds) && otherArea > thisArea) {
			largerOverlapCount++;
		}
	});

	// Each overlap level adds 0.05 units of height
	return largerOverlapCount * 0.05;
}

/**
 * Calculate the maximum z-offset for all groups containing a block
 * This determines how high the block should be positioned
 */
export function calculateBlockZOffset(
	blockName: string,
	layout: LayoutResult,
): number {
	let maxZOffset = 0;

	layout.groups.forEach((group) => {
		if (group.nodeIds.includes(blockName)) {
			const zOffset = calculateGroupZOffset(group, layout.groups);
			maxZOffset = Math.max(maxZOffset, zOffset);
		}
	});

	return maxZOffset;
}

/**
 * Get the Y-axis height for a block (above its region)
 */
export function getBlockHeight(
	blockName: string,
	layout: LayoutResult,
): number {
	const regionZOffset = calculateBlockZOffset(blockName, layout);
	return regionZOffset + 1.2; // 1.2 units above the region
}

/**
 * Get the Y-axis height for an edge (just above the region surface)
 */
export function getEdgeHeight(blockName: string, layout: LayoutResult): number {
	const regionZOffset = calculateBlockZOffset(blockName, layout);
	return regionZOffset + 0.1; // 0.1 units above the region
}

/**
 * Calculate expand factor for a group based on overlaps
 * Larger groups that overlap with smaller groups are expanded
 */
export function calculateGroupExpandFactor(
	group: LayoutGroup,
	allGroups: LayoutGroup[],
): number {
	const thisArea = calculateGroupArea(group);
	const thisBounds = calculateGroupBounds(group);
	let hasOverlap = false;

	allGroups.forEach((otherGroup) => {
		if (group.id === otherGroup.id) return;

		const otherArea = calculateGroupArea(otherGroup);
		const otherBounds = calculateGroupBounds(otherGroup);

		// If this group overlaps with a smaller group, it should expand
		if (boundsOverlap(thisBounds, otherBounds) && otherArea < thisArea) {
			hasOverlap = true;
		}
	});

	return hasOverlap ? 1.15 : 1; // Expand by 15% if overlapping
}

/**
 * Calculate the 4 corner points for a region in world coordinates
 * Includes margin, expansion, and z-offset for overlapping regions
 */
export function calculateRegionCorners(
	group: LayoutGroup,
	allGroups: LayoutGroup[],
	marginPercent = 0.05,
): {
	p1: [number, number, number];
	p2: [number, number, number];
	p3: [number, number, number];
	p4: [number, number, number];
} {
	const { minX, maxX, minY, maxY } = group.boundingBox;
	const width = maxX - minX;
	const height = maxY - minY;

	// Calculate margin (5% of dimensions)
	const marginX = width * marginPercent;
	const marginY = height * marginPercent;

	// Lift regions slightly above the floor while preserving overlap layering.
	const zOffset = calculateGroupZOffset(group, allGroups) + 0.03;

	// Get expand factor (15% expansion for larger overlapping regions)
	const expandFactor = calculateGroupExpandFactor(group, allGroups);

	// Calculate center point for expansion
	const centerX = (minX + maxX) / 2;
	const centerY = (minY + maxY) / 2;

	// Apply margin and expansion
	const expandedWidth = (width + marginX * 2) * expandFactor;
	const expandedHeight = (height + marginY * 2) * expandFactor;
	const expandedMinX = centerX - expandedWidth / 2;
	const expandedMaxX = centerX + expandedWidth / 2;
	const expandedMinY = centerY - expandedHeight / 2;
	const expandedMaxY = centerY + expandedHeight / 2;

	// Convert to world coordinates and apply z-offset
	const p1: [number, number, number] = [
		...toWorldCoordinates(expandedMinX, expandedMinY),
	];
	const p2: [number, number, number] = [
		...toWorldCoordinates(expandedMaxX, expandedMinY),
	];
	const p3: [number, number, number] = [
		...toWorldCoordinates(expandedMaxX, expandedMaxY),
	];
	const p4: [number, number, number] = [
		...toWorldCoordinates(expandedMinX, expandedMaxY),
	];

	// Lift region based on z-offset
	p1[1] += zOffset;
	p2[1] += zOffset;
	p3[1] += zOffset;
	p4[1] += zOffset;

	return { p1, p2, p3, p4 };
}
