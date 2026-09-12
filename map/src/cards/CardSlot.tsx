import type { ReactNode } from "react";
import * as THREE from "three";
import { currentTheme } from "@/theme/theme";
import { Card } from "../components/Card";
import { useCardSlideAnimation } from "./useCardSlideAnimation";

export interface CardSlotConfig {
	components: ReactNode[];
	zOffset: number;
}

export interface CardSlotProps {
	config: CardSlotConfig;
	cardWidth: number;
	cardHeight: number;
	blockColor: string;
	blockHalfWidth: number;
	/** Half-depth of the parent block — used to position the card on the front face when docked */
	blockHalfDepth: number;
	/** Whether this card is the currently active (docked) card — controlled by Block */
	isActive: boolean;
	/** Whether the cursor is over this card's open side slot on the block */
	isSlotHovered: boolean;
	/** Called when this card should also select its parent block. */
	onSelect?: () => void;
	/** Called when the active card's front surface is hovered. */
	onFrontHoverStart?: () => void;
	/** Called when the pointer leaves the active card's front surface. */
	onFrontHoverEnd?: () => void;
}

export function CardSlot({
	config,
	cardWidth,
	cardHeight,
	blockColor,
	blockHalfWidth,
	blockHalfDepth,
	isActive,
	isSlotHovered,
	onSelect,
	onFrontHoverStart,
	onFrontHoverEnd,
}: CardSlotProps) {
	const { cardRef, initialPosition } = useCardSlideAnimation({
		blockHalfWidth,
		blockHalfDepth,
		cardWidth,
		zOffset: config.zOffset,
		isActive,
		isSlotHovered,
	});

	const primaryComponent = config.components[0] ?? null;
	const cardColor = new THREE.Color(blockColor)
		.lerp(
			new THREE.Color(currentTheme.cardHighlightColor),
			isActive ? 0.42 : 0.24,
		)
		.getStyle();

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: THREE.Group is not an HTML element; onClick is a valid R3F pointer event
		<group
			ref={cardRef}
			position={initialPosition}
			onPointerEnter={(e) => {
				if (!isActive) {
					return;
				}
				e.stopPropagation();
				onFrontHoverStart?.();
			}}
			onPointerMove={(e) => {
				if (!isActive) {
					return;
				}
				e.stopPropagation();
				onFrontHoverStart?.();
			}}
			onPointerLeave={(e) => {
				if (!isActive) {
					return;
				}
				e.stopPropagation();
				onFrontHoverEnd?.();
			}}
			onClick={(e) => {
				e.stopPropagation();
				onSelect?.();
			}}
		>
			<Card
				position={[0, 0, config.zOffset]}
				anchor="center"
				width={cardWidth}
				height={cardHeight}
				label={undefined}
				color={cardColor}
				glowColor={blockColor}
				textColor={currentTheme.cardTextColor}
				fontSize={0.12}
			>
				<group position={[0, 0, 0.001]}>{primaryComponent}</group>
			</Card>
		</group>
	);
}
