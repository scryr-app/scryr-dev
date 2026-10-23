import type { ReactNode } from "react";
import * as THREE from "three";
import { currentTheme } from "@/theme/theme";
import { Card } from "../components/Card";
import { useCardFocusAnimation } from "./useCardFocusAnimation";
import { useCardFocus } from "./CardFocusContext";
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
	const { focusedIds } = useCardFocus();
	const primaryComponent = config.components[0] ?? null;
	const cardColor = new THREE.Color(blockColor)
		.lerp(new THREE.Color(currentTheme.cardHighlightColor), isActive ? 0.42 : 0.24)
		.getStyle();

	return (
		// biome-ignore lint/a11y/noStaticElementInteractions: THREE.Group is not an HTML element; onClick is a valid R3F pointer event
		<group
			ref={cardRef}
			position={initialPosition}
			onClick={(e) => {
				e.stopPropagation();
				if (!focusedIds.length) onSelect?.();
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
				<Card
					position={[0, 0, 0]}
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
		</group>
	);
}
