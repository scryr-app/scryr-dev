import { Text } from "@react-three/drei/core/Text";
import { useMemo } from "react";
import { Block } from "@/block";
import { CameraController } from "@/camera";
import type { Block as GraphqlBlock } from "@/graphql/generated";
import { useSelectedMap } from "@/graphql/sampleStore";
import { useBlocksData } from "@/graphql/useBlocksData";
import { currentTheme } from "@/theme/theme";
import { Line } from "./Line";
import {
	calculateRegionCorners,
	getBlockHeight,
	getEdgeHeight,
	getEdgeWorldPath,
	type LayoutResult,
	toWorldCoordinates,
} from "./layout";
import { MapFloor } from "./MapFloor";
import { darkenHexColor, getRegionColor } from "./mapColors";
import { Region, Sign } from "./Region";
import { useMapLayout } from "./useMapLayout";

interface MapStatusTextProps {
	children: string;
	fontSize?: number;
}

function MapStatusText({ children, fontSize = 0.4 }: MapStatusTextProps) {
	return (
		<Text color={currentTheme.errorColor} fontSize={fontSize}>
			{children}
		</Text>
	);
}

function MapRegions({ layout }: { layout: LayoutResult }) {
	return (
		<>
			{layout.groups.map((group, groupIndex) => {
				const { p1, p2, p3, p4 } = calculateRegionCorners(group, layout.groups);
				const groupColor = getRegionColor(group.tag);

				return (
					<Region
						key={`group-${group.id}`}
						p1={p1}
						p2={p2}
						p3={p3}
						p4={p4}
						color={groupColor}
						labelColor={darkenHexColor(groupColor)}
						zIndex={groupIndex}
					/>
				);
			})}
		</>
	);
}

function MapRegionSigns({ layout }: { layout: LayoutResult }) {
	return (
		<>
			{layout.groups.map((group) => {
				const worldPos = toWorldCoordinates(
					group.signPosition.x,
					group.signPosition.y,
				);
				const signColor = getRegionColor(group.tag);

				return (
					<group
						key={`sign-${group.id}`}
						position={[worldPos[0], 0.28, worldPos[2]]}
					>
						<Sign
							label={group.tag}
							color={signColor}
							labelColor={darkenHexColor(signColor)}
							fontSize={0.28}
							margin={0.14}
						/>
					</group>
				);
			})}
		</>
	);
}

function MapEdges({ layout }: { layout: LayoutResult }) {
	return (
		<>
			{layout.edges.map((edge) => {
				const pathPoints = getEdgeWorldPath(edge.id, layout);
				if (!pathPoints || pathPoints.length < 2) {
					return null;
				}

				const sourceHeight = edge.source
					? getEdgeHeight(edge.source, layout)
					: 0.1;
				const targetHeight = edge.target
					? getEdgeHeight(edge.target, layout)
					: 0.1;

				return pathPoints.slice(0, -1).map((start, index) => {
					const end = pathPoints[index + 1];
					if (!end) {
						return null;
					}

					const startLifted: [number, number, number] = [
						start[0],
						start[1] + sourceHeight,
						start[2],
					];
					const endLifted: [number, number, number] = [
						end[0],
						end[1] + targetHeight,
						end[2],
					];

					return (
						<Line
							key={`edge-${edge.id}-${start.join(",")}-${end.join(",")}`}
							start={startLifted}
							end={endLifted}
							color={currentTheme.connectionColor}
							thickness={0.04}
						/>
					);
				});
			})}
		</>
	);
}

interface MapBlocksProps {
	blocks: GraphqlBlock[];
	layout: LayoutResult;
}

function MapBlocks({ blocks, layout }: MapBlocksProps) {
	const nodeById = useMemo(
		() => new Map(layout.nodes.map((node) => [node.id, node])),
		[layout.nodes],
	);
	const diagramTagsByBlockName = useMemo(() => {
		const tagsByBlockName = new Map<string, string[]>();

		for (const group of layout.groups) {
			for (const nodeId of group.nodeIds) {
				const tags = tagsByBlockName.get(nodeId) ?? [];
				tags.push(group.tag);
				tagsByBlockName.set(nodeId, tags);
			}
		}

		return tagsByBlockName;
	}, [layout.groups]);

	return (
		<>
			{blocks.map((block, index) => {
				const blockName = block.name ?? "";
				const layoutNode = nodeById.get(blockName);
				if (!layoutNode) {
					return null;
				}

				const worldPos = toWorldCoordinates(
					layoutNode.x + layoutNode.width / 2,
					layoutNode.y + layoutNode.height / 2,
				);
				const blockHeight = getBlockHeight(blockName, layout);
				const blockDiagrams = diagramTagsByBlockName.get(blockName) ?? [];

				return (
					<Block
						key={blockName || `building-${layoutNode.x}-${layoutNode.y}`}
						position={[worldPos[0], blockHeight, worldPos[2]]}
						color={currentTheme.getColorByIndex(index)}
						name={blockName}
						icon={block.icon || ""}
						classification={block.consumerType || undefined}
						description={block.description || undefined}
						version={block.version || undefined}
						language={block.language || undefined}
						frameworks={block.frameworks}
						deployment={block.deployment || undefined}
						ownerTeam={block.ownerTeam || undefined}
						authType={block.authType || undefined}
						iacTool={block.iacTool || undefined}
						monitoring={block.monitoring || undefined}
						tracing={block.tracing || undefined}
						logAggregation={block.logAggregation || undefined}
						cicdTool={block.cicdTool || undefined}
						docs={block.docs}
						links={block.links}
						diagrams={blockDiagrams}
						blockData={block}
					/>
				);
			})}
		</>
	);
}

export function MapDisplay() {
	const selectedMap = useSelectedMap();
	const { blocks, error, isLoading } = useBlocksData(
		selectedMap.id
			? { scryIdentifier: selectedMap.id }
			: { sample: selectedMap.key },
	);
	const { layout, layoutError } = useMapLayout(blocks);

	if (error) {
		console.error("Error fetching blocks:", error);
		const message =
			error instanceof Error ? error.message : "Failed to load map data.";
		return <MapStatusText>{message}</MapStatusText>;
	}

	if (layoutError) {
		return (
			<MapStatusText fontSize={0.5}>
				{`Layout Error: ${layoutError.message}`}
			</MapStatusText>
		);
	}

	if (isLoading || !layout) {
		return <Text>Loading</Text>;
	}

	return (
		<>
			<CameraController />
			<MapFloor />
			<MapRegions layout={layout} />
			<MapRegionSigns layout={layout} />
			<MapEdges layout={layout} />
			<MapBlocks blocks={blocks} layout={layout} />
		</>
	);
}
