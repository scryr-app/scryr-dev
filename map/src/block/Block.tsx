import { useEffect, useRef, useState } from "react";
import { cameraStore } from "@/camera";
import { InfoCard, OtherDiagramCard } from "@/cards";
import { getBlockCardData } from "@/cards/blockCardData";
import { useMapTray } from "@/cards/MapTrayContext";
import type { Block as GraphqlBlock } from "@/graphql/generated";
import { currentTheme } from "@/theme/theme";
import { CardSlots } from "./CardSlots";
import { type BlockCardGroup, createBlockDataCards } from "./defaultCards";
import { TopLabel } from "./TopLabel";
import { Walls } from "./Walls";
import { getCardHeaderZoomX, ZoomButton } from "./ZoomButton";

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
	cicdTool?: string;
	docs?: string[];
	links?: BlockLink[];
	diagrams?: string[];
	blockData?: GraphqlBlock;
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
	width = 3,
	height = 2,
	depth = 1,
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
	const blockFocusId = blockData?.name ?? name;
	const [isZoomButtonVisible, setIsZoomButtonVisible] = useState(false);
	const zoomHideTimer = useRef<number | null>(null);
	const fallbackCards =
		cards ?? Array.from({ length: 8 }, () => ({ components: [] }));
	const cardData = blockData ? getBlockCardData(blockData) : null;
	const derivedCards = cardData
		? createBlockDataCards(cardData)
		: fallbackCards;

	/** Global card selection from the bottom MapTray */
	const { activeCardIndex, selectBlock } = useMapTray();
	const handleBlockSelect = () => {
		if (!blockData?.name) {
			return;
		}

		selectBlock(blockData.name, blockData.lineNumber ?? null);
	};

	const CARD_MARGIN = 0.1; // world-unit margin around all edges
	const cardWidth = width - 2 * CARD_MARGIN;
	const cardHeight = height - 2 * CARD_MARGIN;
	const zoomButtonX = getCardHeaderZoomX(cardWidth);
	const zoomButtonY = cardHeight / 2 - 0.13;

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
		{ id: "info", categoryIndex: 0, components: [overviewCard] },
		...derivedCards,
		{
			id: "diagrams",
			categoryIndex: 7,
			components: diagrams.length > 0 ? [otherDiagramCard] : [],
		},
	];

	const showZoomButton = () => {
		if (zoomHideTimer.current !== null) {
			window.clearTimeout(zoomHideTimer.current);
			zoomHideTimer.current = null;
		}
		if (!isZoomButtonVisible) {
			setIsZoomButtonVisible(true);
		}
	};

	const scheduleZoomButtonHide = () => {
		if (zoomHideTimer.current !== null) {
			window.clearTimeout(zoomHideTimer.current);
		}
		zoomHideTimer.current = window.setTimeout(() => {
			setIsZoomButtonVisible(false);
			zoomHideTimer.current = null;
		}, 180);
	};

	useEffect(() => {
		return () => {
			if (zoomHideTimer.current !== null) {
				window.clearTimeout(zoomHideTimer.current);
			}
		};
	}, []);

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

			<mesh
				position={[0, 0, hd + 0.018]}
				onPointerEnter={(event) => {
					event.stopPropagation();
					showZoomButton();
				}}
				onPointerMove={(event) => {
					event.stopPropagation();
					showZoomButton();
				}}
				onPointerLeave={(event) => {
					event.stopPropagation();
					scheduleZoomButtonHide();
				}}
			>
				<planeGeometry args={[width, height]} />
				<meshBasicMaterial transparent opacity={0} depthWrite={false} />
			</mesh>

			<ZoomButton
				blockId={blockFocusId}
				x={zoomButtonX}
				y={zoomButtonY}
				z={hd + 0.13}
				blockPosition={position}
				blockWidth={width}
				blockHeight={height}
				blockDepth={depth}
				onSelectBlock={handleBlockSelect}
				isVisible={isZoomButtonVisible}
				onHoverChange={(isHovered) => {
					if (isHovered) {
						showZoomButton();
					} else {
						scheduleZoomButtonHide();
					}
				}}
			/>

			{/* Card slots — OverviewCard is always first */}
			<CardSlots
				cards={allCards}
				cardWidth={cardWidth}
				cardHeight={cardHeight}
				blockColor={color}
				blockHalfWidth={hw}
				blockHalfDepth={hd}
				activeCardIndex={activeCardIndex}
				onBlockSelect={handleBlockSelect}
				onFrontFaceHoverStart={showZoomButton}
				onFrontFaceHoverEnd={scheduleZoomButtonHide}
			/>
		</group>
	);
}
