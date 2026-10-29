import type { ReactNode } from "react";
import * as THREE from "three";
import { currentTheme } from "@/theme/theme";
import { CardFrame } from "../components/CardFrame";
import { useCardFocusAnimation } from "./useCardFocusAnimation";
import { useCardFocus } from "./CardFocusContext";
import { CardSurface } from "./CardSurface";
import { useCardSlideAnimation } from "./useCardSlideAnimation";

export interface CardSlotConfig {
	components: ReactNode[];
	zOffset: number;
}

export interface CardSlotProps {
	focusId: string;
	config: CardSlotConfig;
	cardWidth: number;
	cardHeight: number;
	blockColor: string;
	blockHalfWidth: number;
	/** Half-depth of the parent block — used to position the card on the front face when docked */
	blockHalfDepth: number;
	/** Whether this card is the currently active (docked) card — controlled by Block */
	isActive: boolean;
	/** Whether the pointer is over the slot or its exposed card. */
	isSlotHovered: boolean;
	/** Called when this card should also select its parent block. */
	onSelect?: () => void;
}

export function CardSlot({
	focusId,
	config,
	cardWidth,
	cardHeight,
	blockColor,
	blockHalfWidth,
	blockHalfDepth,
	isActive,
	isSlotHovered,
	onSelect,
}: CardSlotProps) {
	const { cardRef, initialPosition } = useCardSlideAnimation({
		blockHalfWidth,
		blockHalfDepth,
		cardWidth,
		zOffset: config.zOffset,
		isActive,
		isSlotHovered,
	});

	const focusRef = useCardFocusAnimation(focusId, cardWidth, cardHeight, config.zOffset);
	const { focusedIds, isSelecting, focusCard } = useCardFocus();
	const primaryComponent = config.components[0] ?? null;
	const cardColor = new THREE.Color(blockColor)
		.lerp(new THREE.Color(currentTheme.cardHighlightColor), isActive ? 0.42 : 0.24)
		.getStyle();

	const { cards: appearance, textures } = currentTheme.appearance;
	const surfaceColor = new THREE.Color(
		textures.paperColor ?? (appearance.useBlockColor ? blockColor : cardColor),
	)
		.multiplyScalar(appearance.brightness)
		.getStyle();

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: THREE.Group is not an HTML element; onClick is a valid R3F pointer event
		<group
			ref={cardRef}
			position={initialPosition}
			onClick={(e) => {
				e.stopPropagation();
				if (isSelecting) focusCard(focusId);
				else if (!focusedIds.length) onSelect?.();
			}}
		>
			<group
				ref={focusRef}
				position={[0, 0, config.zOffset]}
				name={`focus-card:${focusId}`}
				onPointerOver={(event) => {
					if (focusedIds.length) event.stopPropagation();
				}}
			>
				{isSelecting && (
					<mesh
						position={[0, 0, 0.12]}
						name={`select-card:${focusId}`}
						onClick={(event) => {
							event.stopPropagation();
							focusCard(focusId);
						}}
					>
						<planeGeometry args={[cardWidth, cardHeight]} />
						<meshBasicMaterial
							color="#93c5fd"
							transparent
							opacity={isSlotHovered ? 0.2 : 0.09}
							depthWrite={false}
							side={THREE.DoubleSide}
						/>
					</mesh>
				)}
				<CardFrame
					position={[0, 0, 0]}
					anchor="center"
					width={cardWidth}
					height={cardHeight}
					color={cardColor}
					glowColor={blockColor}
				>
					<CardSurface
						width={cardWidth}
						height={cardHeight}
						focusId={focusId}
						color={surfaceColor}
						titleInteractive={!focusedIds.length && (isActive || isSlotHovered)}
					>
						{primaryComponent}
					</CardSurface>
				</CardFrame>
			</group>
		</group>
	);
}
