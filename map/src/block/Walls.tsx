import * as THREE from "three";
import { GlowFrame } from "@/components/GlowFrame";
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
	const { walls, innerWalls, shapes } = currentTheme.appearance;
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
					{...walls}
					ior={1.48}
					thickness={0.35}
					attenuationColor={color}
					attenuationDistance={1.6}
					emissive={glowColor}
					emissiveMap={glowTexture ?? undefined}
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
					{...innerWalls}
					emissive={glowColor}
				/>
			</mesh>
			{/* Two lit bevels reveal the depth of the dark glass casing. */}
			{shapes.frameFront > 0 && (
				<group position={[0, 0, hd + 0.012]}>
					<GlowFrame
						width={hw * 2}
						height={hh * 2}
						color={color}
						strength={shapes.frameFront}
					/>
				</group>
			)}
			{shapes.frameBack > 0 && (
				<group position={[0, 0, -hd - 0.012]}>
					<GlowFrame
						width={hw * 2}
						height={hh * 2}
						color={color}
						strength={shapes.frameBack}
					/>
				</group>
			)}
		</>
	);
}
