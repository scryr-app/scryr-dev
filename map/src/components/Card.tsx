import { RoundedBox } from "@react-three/drei/core/RoundedBox";
import { Text } from "@react-three/drei/core/Text";
import { useRef } from "react";
import type { Group } from "three";
import * as THREE from "three";

import { useCrystalGlowTexture } from "@/theme/textures";

const CARD_DEPTH = 0.025;

export interface CardProps {
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
	/** Card depth (thickness) */
	depth?: number;
	/** Border radius */
	borderRadius?: number;
	/** Content to render on the card face (can be text, components, etc.) */
	children?: React.ReactNode;
	/** Optional label at the top of the card */
	label?: string;
	/** Text color for label */
	textColor?: string;
	/** Font size for label */
	fontSize?: number;
}

/**
 * 3D Card Component for React Three Fiber
 * Renders a thin card with rounded edges, slight depth, and customizable content
 * Perfect for displaying information in 3D scenes
 */
export function Card({
	position,
	anchor = "bottom",
	color = "#f5f3f0",
	glowColor = color,
	width = 40,
	height = 3,
	children,
	label,
	textColor = "#2d2421",
	fontSize = 0.3,
}: CardProps) {
	const groupRef = useRef<Group>(null);
	const baseColor = new THREE.Color(color);
	const glowTexture = useCrystalGlowTexture();
	const rimColor = baseColor.clone().lerp(new THREE.Color("#ffffff"), 0.22);
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
			<RoundedBox
				args={[width, height, CARD_DEPTH]}
				radius={CARD_DEPTH / 2}
				smoothness={8}
				position={[0, 0, 0]}
			>
				<meshPhysicalMaterial
					color={baseColor}
					metalness={0.03}
					roughness={0.55}
					clearcoat={0.18}
					clearcoatRoughness={0.5}
					envMapIntensity={0.18}
					emissive={glowColor}
					emissiveMap={glowTexture ?? undefined}
					emissiveIntensity={0.28}
				/>
			</RoundedBox>

			<RoundedBox
				args={[width - 0.06, height - 0.06, CARD_DEPTH * 0.24]}
				radius={CARD_DEPTH / 1.8}
				smoothness={6}
				position={[0, 0, CARD_DEPTH / 2 + 0.002]}
			>
				<meshStandardMaterial
					color={rimColor}
					transparent
					opacity={0.12}
					metalness={0.08}
					roughness={0.68}
				/>
			</RoundedBox>

			{/* Optional label on top */}
			{label && (
				<Text
					position={[0, height / 2, CARD_DEPTH / 2 + 0.05]}
					fontSize={fontSize}
					color={textColor}
					anchorX="center"
					anchorY="top"
					maxWidth={width - 0.2}
				>
					{label}
				</Text>
			)}

			{/* Content container - rendered on the front face */}
			{children && (
				<group position={[0, 0, CARD_DEPTH / 2 + 0.05]}>{children}</group>
			)}
		</group>
	);
}
