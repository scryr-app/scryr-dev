import { Text } from "@react-three/drei/core/Text";
import { useRef } from "react";
import type { Group } from "three";
import * as THREE from "three";
import { currentTheme } from "@/theme/theme";

export interface LineProps {
	/** Starting position [x, y?] - y defaults to 0 if omitted, z defaults to 0 */
	start: [number] | [number, number] | [number, number, number];
	/** Ending position [x, y?] - y defaults to 0 if omitted, z defaults to 0 */
	end: [number] | [number, number] | [number, number, number];
	/** Line color (hex or CSS color) */
	color?: string;
	/** Line thickness */
	thickness?: number;
	/** Optional label that floats above the line */
	label?: string;
	/** Text color for label */
	labelColor?: string;
	/** Font size for label */
	labelSize?: number;
}

/**
 * 3D Line Component for React Three Fiber
 * Renders a cylindrical line between two 3D points
 * Perfect for connecting or dividing sections in 3D scenes
 */
export function Line({
	start,
	end,
	color = currentTheme.connectionColor,
	thickness = 0.05,
	label,
	labelColor = currentTheme.fontColor,
	labelSize = 0.2,
}: LineProps) {
	const { connections } = currentTheme.appearance;
	const groupRef = useRef<Group>(null);

	// Normalize positions to 3D coordinates (y defaults to 0, z defaults to 0)

	const startPos = new THREE.Vector3(
		start[0],
		start[2] ? start[1] : 0,
		start[2] ? start[2] : start[1],
	);
	const endPos = new THREE.Vector3(
		end[0],
		end[2] ? end[1] : 0,
		end[2] ? end[2] : end[1],
	);

	// Calculate direction and length
	const direction = endPos.clone().sub(startPos);
	const length = direction.length();
	const midpoint = startPos.clone().add(endPos).multiplyScalar(0.5);

	// Calculate rotation to align cylinder (built along +Y) with the direction vector
	const quaternion = new THREE.Quaternion();
	if (length > 1e-6) {
		const lookAtMatrix = new THREE.Matrix4().lookAt(
			startPos,
			endPos,
			new THREE.Vector3(0, 1, 0),
		);
		const lookAtQuaternion = new THREE.Quaternion().setFromRotationMatrix(
			lookAtMatrix,
		);
		const alignYAxis = new THREE.Quaternion().setFromAxisAngle(
			new THREE.Vector3(1, 0, 0),
			-Math.PI / 2,
		);
		quaternion.copy(lookAtQuaternion).multiply(alignYAxis);
	}

	// Rotate only the text: 90° upward in line space, then by the line's conjugate
	const textQuaternion = quaternion
		.clone()
		.multiply(
			new THREE.Quaternion().setFromAxisAngle(
				new THREE.Vector3(1, 0, 0),
				Math.PI / 2,
			),
		)
		.multiply(
			new THREE.Quaternion().setFromAxisAngle(
				new THREE.Vector3(0, 1, 0),
				Math.PI / 2,
			),
		);

	return (
		<group
			ref={groupRef}
			position={midpoint.toArray() as [number, number, number]}
		>
			{/* 3D cylinder line */}
			<mesh quaternion={quaternion}>
				<cylinderGeometry args={[thickness / 2, thickness / 2, length, 16]} />
				{connections.luminous ? (
					<meshBasicMaterial color={color} toneMapped={false} />
				) : (
					<meshStandardMaterial color={color} metalness={0.3} roughness={0.6} />
				)}
			</mesh>
			{/* A restrained halo follows the actual connection path. */}
			{connections.haloOpacity > 0 && (
				<mesh quaternion={quaternion} raycast={() => {}}>
					<cylinderGeometry
						args={[thickness * 1.8, thickness * 1.8, length, 12]}
					/>
					<meshBasicMaterial
						color={color}
						transparent
						opacity={connections.haloOpacity}
						depthWrite={false}
						blending={THREE.AdditiveBlending}
						toneMapped={false}
					/>
				</mesh>
			)}
			{label && (
				<Text
					quaternion={textQuaternion}
					position={[0, 0.2, 0]}
					fontSize={labelSize}
					color={labelColor}
					anchorX="center"
					anchorY="middle"
				>
					{label}
				</Text>
			)}
		</group>
	);
}
