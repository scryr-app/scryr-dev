import { useFrame } from "@react-three/fiber";
import { useEffect, useRef, useState } from "react";
import * as THREE from "three";

/** Matches CARD_DEPTH in Card.tsx */
const CARD_DEPTH = 0.025;
/** Tiny z-offset so the docked card sits flush but doesn't z-fight the face */
const DOCK_GAP = 0.0001;
/** Distance threshold to consider a lerp "arrived" and advance to next phase */
const NEAR = 0.008;

/**
 * Animation phases:
 *
 * Docking sequence (click while in slot):
 *   slot → dock-out → dock-fwd → dock-in → docked
 *
 * Undocking sequence (click while docked):
 *   docked → undock-right → undock-back → undock-slot → slot
 */
type AnimPhase =
	| "slot"
	| "dock-out"
	| "dock-fwd"
	| "dock-in"
	| "docked"
	| "undock-right"
	| "undock-back"
	| "undock-slot";

interface UseCardSlideAnimationProps {
	/** Half-width of the block (hw) */
	blockHalfWidth: number;
	/** Half-depth of the block (hd) — needed to dock card on the front face */
	blockHalfDepth: number;
	/** Width of the card */
	cardWidth: number;
	/** z-offset baked into the Card's position inside this slot */
	zOffset: number;
	/** Animation speed multiplier (default: 8) */
	animationSpeed?: number;
	/** Whether this card should be docked to the front face (controlled by Block) */
	isActive: boolean;
	/** Whether the cursor is over this card's open side slot on the block */
	isSlotHovered: boolean;
}

interface UseCardSlideAnimationReturn {
	/** Ref to attach to the card group */
	cardRef: React.RefObject<THREE.Group | null>;
	/** Initial position for the card group */
	initialPosition: [number, number, number];
	/** Whether the card is currently docked to the front face */
	isDocked: boolean;
}

/**
 * Animates a card through side-slot hover and toolbar-controlled docking.
 *
 * Dock   : slide out right → move forward → slide back to front face centre
 * Undock : slide right → move back → slide into slot
 */
export function useCardSlideAnimation({
	blockHalfWidth,
	blockHalfDepth,
	cardWidth,
	zOffset,
	animationSpeed = 8,
	isActive,
	isSlotHovered,
}: UseCardSlideAnimationProps): UseCardSlideAnimationReturn {
	const [phase, setPhase] = useState<AnimPhase>("slot");
	const cardRef = useRef<THREE.Group>(null);

	// React to external isActive changes to drive the dock/undock animation
	useEffect(() => {
		if (isActive) {
			// Start docking unless already headed that way
			setPhase((p) =>
				p === "docked" ||
				p === "dock-in" ||
				p === "dock-fwd" ||
				p === "dock-out"
					? p
					: "dock-out",
			);
		} else {
			// Start undocking unless already headed that way
			setPhase((p) =>
				p === "slot" ||
				p === "undock-slot" ||
				p === "undock-back" ||
				p === "undock-right"
					? p
					: "undock-right",
			);
		}
	}, [isActive]);

	const cardHalfWidth = cardWidth / 2;
	const slotX = blockHalfWidth - cardHalfWidth;
	const slideOutX = blockHalfWidth + cardHalfWidth;
	// Counter-balance zOffset so the card's back face lands flush on the block front face
	const dockedZ = blockHalfDepth - zOffset + CARD_DEPTH / 2 + DOCK_GAP;

	const posX = useRef(slotX);
	const posZ = useRef(0);

	useFrame((_state, delta) => {
		if (!cardRef.current) return;

		// Keep interpolation bounded even after a slow frame or a suspended tab.
		const speed = 1 - Math.exp(-delta * animationSpeed);

		switch (phase) {
			// ── Docking ──────────────────────────────────────────────────────
			case "dock-out": {
				posX.current = THREE.MathUtils.lerp(posX.current, slideOutX, speed);
				if (Math.abs(posX.current - slideOutX) < NEAR) {
					posX.current = slideOutX;
					setPhase("dock-fwd");
				}
				break;
			}
			case "dock-fwd": {
				posZ.current = THREE.MathUtils.lerp(posZ.current, dockedZ, speed);
				if (Math.abs(posZ.current - dockedZ) < NEAR) {
					posZ.current = dockedZ;
					setPhase("dock-in");
				}
				break;
			}
			case "dock-in": {
				posX.current = THREE.MathUtils.lerp(posX.current, 0, speed);
				if (Math.abs(posX.current) < NEAR) {
					posX.current = 0;
					setPhase("docked");
				}
				break;
			}

			// ── Undocking ────────────────────────────────────────────────────
			case "undock-right": {
				posX.current = THREE.MathUtils.lerp(posX.current, slideOutX, speed);
				if (Math.abs(posX.current - slideOutX) < NEAR) {
					posX.current = slideOutX;
					setPhase("undock-back");
				}
				break;
			}
			case "undock-back": {
				posZ.current = THREE.MathUtils.lerp(posZ.current, 0, speed);
				if (Math.abs(posZ.current) < NEAR) {
					posZ.current = 0;
					setPhase("undock-slot");
				}
				break;
			}
			case "undock-slot": {
				posX.current = THREE.MathUtils.lerp(posX.current, slotX, speed);
				if (Math.abs(posX.current - slotX) < NEAR) {
					posX.current = slotX;
					setPhase("slot");
				}
				break;
			}

			// ── Idle states ──────────────────────────────────────────────────
			case "slot": {
				const targetX = isSlotHovered ? slideOutX : slotX;
				posX.current = THREE.MathUtils.lerp(posX.current, targetX, speed);
				break;
			}
			case "docked":
				// Already at rest; nothing to lerp
				break;
		}

		cardRef.current.position.x = posX.current;
		cardRef.current.position.z = posZ.current;
	});

	return {
		cardRef,
		initialPosition: [slotX, 0, 0],
		isDocked: phase === "docked",
	};
}
