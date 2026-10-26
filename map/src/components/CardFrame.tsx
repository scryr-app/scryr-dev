import { RoundedBox } from "@react-three/drei/core/RoundedBox";
import { useEffect, useMemo, useRef } from "react";
import type { Group } from "three";
import * as THREE from "three";
import { RoundedBoxGeometry } from "three-stdlib";

import { GlowFrame } from "@/components/GlowFrame";
import { useCardTexture, useCrystalGlowTexture } from "@/theme/textures";
import { currentTheme } from "@/theme/theme";

const CARD_DEPTH = 0.025;

export interface CardFrameProps {
	/** Position in 3D space [x, y, z]. Interpreted per `anchor`. */
	position: [number, number, number];
	/** Where the `position` is anchored relative to the card */
	anchor?: "bottom" | "center";
	/** Card color (hex or CSS color) */
	color?: string;
	/** Emission tint, supplied from the parent block before its card is lightened. */
	glowColor?: string;
	/** Card width */
	width?: number;
	/** Card height */
	height?: number;
	/** Content to render on the card face (can be text, components, etc.) */
	children?: React.ReactNode;
}

/**
 * Physical frame only. All card content is rendered by the shared DOM CardSurface.
 */
export function CardFrame({
	position,
	anchor = "bottom",
	color = "#f5f3f0",
	glowColor = color,
	width = 40,
	height = 3,
	children,
}: CardFrameProps) {
	const groupRef = useRef<Group>(null);
	const { cards, shapes } = currentTheme.appearance;
	// Drei's extruded RoundedBox uses world-space UVs. A glow texture needs
	// normalized face UVs, otherwise it clamps into a dark lower-left square.
	const surfaceGeometry = useMemo(
		() => new RoundedBoxGeometry(width, height, CARD_DEPTH, 3, shapes.cardRadius),
		[width, height, shapes.cardRadius],
	);
	useEffect(() => () => surfaceGeometry.dispose(), [surfaceGeometry]);
	const { brightness, useBlockColor, overlayOpacity, rimHighlight, ...material } = cards;
	const baseColor = new THREE.Color(
		currentTheme.appearance.textures.paperColor ?? (useBlockColor ? glowColor : color),
	).multiplyScalar(brightness);
	const glowTexture = useCrystalGlowTexture();
	const paperTexture = useCardTexture();
	const rimColor = (useBlockColor ? new THREE.Color(glowColor) : baseColor.clone()).lerp(
		new THREE.Color("#ffffff"),
		rimHighlight,
	);
	const shadowColor = baseColor.clone().multiplyScalar(0.58);

	// Apply anchor offset so `position` refers to desired anchor point
	const [x, y, z] = position;
	const anchoredPosition: [number, number, number] =
		anchor === "bottom" ? [x, y + height / 2, z] : [x, y, z];

	return (
		<group ref={groupRef} position={anchoredPosition}>
			<RoundedBox
				args={[width + 0.02, height + 0.02, CARD_DEPTH * 0.6]}
				radius={CARD_DEPTH / 1.8}
				smoothness={6}
				position={[0, 0, -CARD_DEPTH * 0.35]}
			>
				<meshStandardMaterial
					color={shadowColor}
					transparent
					opacity={0.35}
					metalness={0.05}
					roughness={0.9}
				/>
			</RoundedBox>

			{/* Main card surface */}
			<mesh geometry={surfaceGeometry}>
				<meshPhysicalMaterial
					color={baseColor}
					map={paperTexture}
					{...material}
					emissive={glowColor}
					emissiveMap={glowTexture ?? undefined}
				/>
			</mesh>

			<RoundedBox
				args={[width - 0.06, height - 0.06, CARD_DEPTH * 0.24]}
				radius={CARD_DEPTH / 1.8}
				smoothness={6}
				position={[0, 0, CARD_DEPTH / 2 + 0.002]}
			>
				<meshStandardMaterial
					color={rimColor}
					transparent
					opacity={overlayOpacity}
					metalness={0.08}
					roughness={0.68}
				/>
			</RoundedBox>

			{shapes.frameCard > 0 && (
				<group position={[0, 0, CARD_DEPTH / 2 + 0.015]}>
					<GlowFrame
						width={width - 0.025}
						height={height - 0.025}
						color={glowColor}
						strength={shapes.frameCard}
					/>
				</group>
			)}

			{/* Content container - rendered on the front face */}
			{children && <group position={[0, 0, CARD_DEPTH / 2 + 0.05]}>{children}</group>}
		</group>
	);
}
