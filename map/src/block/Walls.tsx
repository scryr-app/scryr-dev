import * as THREE from "three";
import { useWallTexture } from "@/theme/textures";
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

	return (
		<>
			{/* Outer walls */}
			{/* Outer texture has color baked in; use white to avoid double-multiply */}
			<mesh geometry={geometry} castShadow receiveShadow>
				<meshStandardMaterial
					color={outerTexture ? "#ffffff" : color}
					map={outerTexture ?? undefined}
					side={THREE.DoubleSide}
					metalness={0.24}
					roughness={0.52}
				/>
			</mesh>

			{/* Inner walls - lighter color for depth */}
			<mesh geometry={innerWallsGeometry} castShadow receiveShadow>
				<meshStandardMaterial
					color={new THREE.Color(color).lerp(
						new THREE.Color(currentTheme.innerWallColor),
						0.1,
					)}
					side={THREE.FrontSide}
					metalness={0.16}
					roughness={0.62}
				/>
			</mesh>
		</>
	);
}
