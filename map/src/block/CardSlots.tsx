import { isValidElement, type ReactNode, useState } from "react";
import type { CardSlotConfig } from "@/cards";
import { CardSlot, getCardLayout } from "@/cards";

export interface CardSlotsProps {
	/** Card configuration with components for each card */
	cards: Array<{ components: ReactNode[] }>;
	/** Width of individual cards */
	cardWidth: number;
	/** Height of individual cards */
	cardHeight: number;
	/** Color of the parent block */
	blockColor: string;
	/** Half-width of the parent block for positioning */
	blockHalfWidth: number;
	/** Half-depth of the parent block for front-face docking */
	blockHalfDepth: number;
	/** Index of the currently active (front-face) card, or null for none */
	activeCardIndex: number | null;
	/** Called when any card in the block is clicked. */
	onBlockSelect?: () => void;
	/** Called when the active front-face card is hovered. */
	onFrontFaceHoverStart?: () => void;
	/** Called when the pointer leaves the active front-face card. */
	onFrontFaceHoverEnd?: () => void;
}

/**
 * CardSlots component manages the dynamic card slots on the right side of a Block.
 * Calculates layout and renders individual CardSlot components.
 * Only one card may be docked to the front face at a time (controlled by Block state).
 */
export function CardSlots({
	cards,
	cardWidth,
	cardHeight,
	blockColor,
	blockHalfWidth,
	blockHalfDepth,
	activeCardIndex,
	onBlockSelect,
	onFrontFaceHoverStart,
	onFrontFaceHoverEnd,
}: CardSlotsProps) {
	const cardLayout = getCardLayout(cards);
	const [hoveredSlotIndex, setHoveredSlotIndex] = useState<number | null>(null);

	const getComponentTypeName = (type: unknown): string => {
		if (typeof type === "string") {
			return type;
		}

		if (typeof type === "function") {
			const componentType = type as {
				displayName?: string;
				name?: string;
			};
			return componentType.displayName ?? componentType.name ?? "component";
		}

		if (typeof type === "object" && type !== null && "displayName" in type) {
			const displayName = type.displayName;
			if (typeof displayName === "string") {
				return displayName;
			}
		}

		return "component";
	};

	const getCardSlotKey = (cardConfig: CardSlotConfig): string => {
		const componentIds = cardConfig.components.map((component) => {
			if (!isValidElement(component)) {
				return typeof component;
			}

			if (component.key != null) {
				return String(component.key);
			}

			return getComponentTypeName(component.type);
		});

		return `card-slot-${componentIds.join("-")}-${cardConfig.zOffset}`;
	};

	return (
		<>
			{cardLayout.map((cardConfig, index) => {
				const slotKey = getCardSlotKey(cardConfig);

				return (
					<group key={slotKey}>
						<mesh
							position={[blockHalfWidth + 0.035, 0, cardConfig.zOffset]}
							onPointerEnter={() => setHoveredSlotIndex(index)}
							onPointerLeave={() => {
								setHoveredSlotIndex((current) =>
									current === index ? null : current,
								);
							}}
						>
							<boxGeometry args={[0.07, cardHeight, 0.075]} />
							<meshBasicMaterial transparent opacity={0} depthWrite={false} />
						</mesh>
						<CardSlot
							config={cardConfig}
							cardWidth={cardWidth}
							cardHeight={cardHeight}
							blockColor={blockColor}
							blockHalfWidth={blockHalfWidth}
							blockHalfDepth={blockHalfDepth}
							isActive={activeCardIndex === index}
							isSlotHovered={hoveredSlotIndex === index}
							onSelect={onBlockSelect}
							onFrontHoverStart={onFrontFaceHoverStart}
							onFrontHoverEnd={onFrontFaceHoverEnd}
						/>
					</group>
				);
			})}
		</>
	);
}
