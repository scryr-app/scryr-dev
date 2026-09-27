import { useEffect, useState } from "react";
import { cameraStore } from "@/camera";
import { InfoCard, OtherDiagramCard } from "@/cards";
import { getBlockCardData } from "@/cards/blockCardData";
import { useMapTray } from "@/cards/MapTrayContext";
import type { GetBlocksQuery } from "@/graphql/generated";
import { currentTheme } from "@/theme/theme";
import { CardSlots } from "./CardSlots";
import { type BlockCardGroup, createBlockDataCards } from "./defaultCards";
import { BLOCK_DIMENSIONS } from "./dimensions";
import { TopLabel } from "./TopLabel";
import { Walls } from "./Walls";
import { ZoomButton } from "./ZoomButton";

export interface BlockLink {
	siteName?: string | null;
	httpUrl?: string | null;
}

export interface BlockProps {
	position?: [number, number, number];
	color?: string;
	name?: string;
	icon?: string;
	fontColor?: string;
	width?: number;
	height?: number;
	depth?: number;
	cards?: BlockCardGroup;
	description?: string;
	classification?: string;
	version?: string;
	language?: string;
	frameworks?: string[];
	deployment?: string;
	ownerTeam?: string;
	authType?: string;
	iacTool?: string;
	monitoring?: string;
	tracing?: string;
	logAggregation?: string;
	docs?: string[];
	links?: BlockLink[];
	diagrams?: string[];
	blockData?: GetBlocksQuery["blocks"][number];
}

/**
 * Block component represents a 3D building/service in the architecture map.
 * Features:
 * - Custom 5-face geometry (no right face for card slots)
 * - Inner walls with lighter color for depth
 * - Top label with icon and name
 * - Dynamic card slots on the right side
 * - Front-facing metadata display
 */
export function Block({
	position = [0, 0, 0],
	color = currentTheme.surface,
	name = "",
	icon = "",
	fontColor = currentTheme.fontColor,
	width = BLOCK_DIMENSIONS.width,
	height = BLOCK_DIMENSIONS.height,
	depth = BLOCK_DIMENSIONS.depth,
	cards,
	description,
	classification,
	version,
	language,
	frameworks = [],
	deployment,
	ownerTeam,
	authType,
	iacTool,
	monitoring,
	tracing,
	logAggregation,
	docs = [],
	links = [],
	diagrams = [],
	blockData,
}: BlockProps) {
	const hw = width / 2;
	const hh = height / 2;
	const hd = depth / 2;
	const [isHovered, setIsHovered] = useState(false);
	const blockFocusId = blockData?.name ?? name;
	const fallbackCards =
		cards ?? Array.from({ length: 6 }, () => ({ components: [] }));
	const cardData = blockData ? getBlockCardData(blockData) : null;
	const derivedCards = cardData
		? createBlockDataCards(cardData)
		: fallbackCards;

	/** Individual card choices override the shared toolbar selection. */
	const { getActiveCardIndex, selectCardForBlock, selectBlock } = useMapTray();
	const activeCardIndex = getActiveCardIndex(blockFocusId);
	const handleBlockSelect = () => {
		if (!blockData?.name) {
			return;
		}

		selectBlock(blockData.name, blockData.lineNumber ?? null);
	};

	const CARD_MARGIN = 0.1; // world-unit margin around all edges
	const cardWidth = width - 2 * CARD_MARGIN;
	const cardHeight = height - 2 * CARD_MARGIN;

	const overviewCard = (
		<InfoCard
			key="info-card"
			classification={classification ?? blockData?.consumerType ?? undefined}
			description={description}
			version={version}
			language={language}
			frameworks={frameworks}
			deployment={deployment}
			ownerTeam={ownerTeam}
			authType={authType}
			iacTool={iacTool ?? blockData?.iacTool ?? undefined}
			monitoring={monitoring}
			tracing={tracing}
			logAggregation={logAggregation}
			docs={docs}
			links={links}
		/>
	);

	const otherDiagramCard = (
		<OtherDiagramCard key="other-diagram-card" diagrams={diagrams} />
	);

	const allCards = [
		{ components: [overviewCard] },
		...derivedCards,
		{ components: diagrams.length > 0 ? [otherDiagramCard] : [] },
	];

	useEffect(() => {
		if (!blockFocusId) {
			return;
		}

		return cameraStore.registerBlock({
			id: blockFocusId,
			position,
			width,
			height,
			depth,
		});
	}, [blockFocusId, position, width, height, depth]);

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: THREE.Group is not an HTML element; onClick is a valid R3F pointer event
		<group
			position={position as [number, number, number]}
			onPointerOver={(event) => {
				event.stopPropagation();
				setIsHovered(true);
			}}
			onPointerOut={() => setIsHovered(false)}
			onClick={() => {
				handleBlockSelect();
			}}
		>
			{/* Top label with icon and name */}
			<TopLabel
				icon={icon}
				name={name}
				fontColor={fontColor}
				hh={hh}
				width={width}
			/>

			{/* Outer and inner walls */}
			<Walls color={color} hw={hw} hh={hh} hd={hd} />

			<ZoomButton
				blockId={blockFocusId}
				isVisible={isHovered}
				blockPosition={position}
				blockWidth={width}
				blockHeight={height}
				blockDepth={depth}
				onSelectBlock={handleBlockSelect}
			/>

			{/* Card slots — OverviewCard is always first */}
			<CardSlots
				blockId={blockFocusId}
				cards={allCards}
				cardWidth={cardWidth}
				cardHeight={cardHeight}
				blockColor={color}
				blockHalfWidth={hw}
				blockHalfDepth={hd}
				activeCardIndex={activeCardIndex}
				onCardSelect={(index) => {
					selectCardForBlock(blockFocusId, index);
					handleBlockSelect();
				}}
			/>
		</group>
	);
}
