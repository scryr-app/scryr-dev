import * as THREE from "three";
import { useCrystalGlowTexture, useWallTexture } from "@/theme/textures";
import { currentTheme } from "@/theme/theme";
import { useBlockGeometry, useInnerWallsGeometry } from "./geometry";

export interface WallsProps {
	/** Main color of the block */
	color: string;
	/** Half-width of the block for geometry calculation */
	hw: number;
	/** Half-height of the block for geometry calculation */
	hh: number;
	/** Half-depth of the block for geometry calculation */
	hd: number;
}

/**
 * Walls component renders both the outer and inner walls of a Block.
 * - Outer walls: main block color with double-sided rendering
 * - Inner walls: slightly lighter color for depth effect
 */
export function Walls({ color, hw, hh, hd }: WallsProps) {
	const geometry = useBlockGeometry({ hw, hh, hd });
	const innerWallsGeometry = useInnerWallsGeometry({ hw, hh, hd });
	const outerTexture = useWallTexture(color);
	const glowTexture = useCrystalGlowTexture();
	const glowColor = new THREE.Color(color);

	return (
		<>
			{/* Outer walls */}
			{/* Outer texture has color baked in; use white to avoid double-multiply */}
			<mesh geometry={geometry} castShadow receiveShadow>
				<meshPhysicalMaterial
					color={outerTexture ? "#ffffff" : color}
					map={outerTexture ?? undefined}
					side={THREE.DoubleSide}
					metalness={0.04}
					roughness={0.52}
					clearcoat={0.22}
					clearcoatRoughness={0.48}
					ior={1.48}
					transmission={0.03}
					thickness={0.35}
					attenuationColor={color}
					attenuationDistance={1.6}
					envMapIntensity={0.2}
					emissive={glowColor}
					emissiveMap={glowTexture ?? undefined}
					emissiveIntensity={0.7}
				/>
			</mesh>

			{/* Inner walls - lighter color for depth */}
			<mesh geometry={innerWallsGeometry} castShadow receiveShadow>
				<meshPhysicalMaterial
					color={new THREE.Color(color).lerp(
						new THREE.Color(currentTheme.innerWallColor),
						0.1,
					)}
					side={THREE.FrontSide}
					metalness={0.02}
					roughness={0.55}
					clearcoat={0.15}
					emissive={glowColor}
					emissiveIntensity={0.32}
				/>
			</mesh>
		</>
	);
}
