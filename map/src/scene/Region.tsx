import { Text } from "@react-three/drei/core/Text";
import { useMemo, useRef } from "react";
import type { Group } from "three";
import * as THREE from "three";
import { useGroundTexture } from "@/theme/textures";

export interface RegionProps {
	/** Top-left corner [x, y?, z?] - y defaults to 0, z defaults to 0 */
	p1: [number] | [number, number] | [number, number, number];
	/** Top-right corner [x, y?, z?] - y defaults to 0, z defaults to 0 */
	p2: [number] | [number, number] | [number, number, number];
	/** Bottom-right corner [x, y?, z?] - y defaults to 0, z defaults to 0 */
	p3: [number] | [number, number] | [number, number, number];
	/** Bottom-left corner [x, y?, z?] - y defaults to 0, z defaults to 0 */
	p4: [number] | [number, number] | [number, number, number];
	/** Plane color (hex or CSS color) */
	color?: string;
	/** Optional label that floats above the plane */
	label?: string;
	/** Text color for label */
	labelColor?: string;
	/** Font size for label */
	labelSize?: number;
	/** Transparency (0-1) */
	opacity?: number;
	/** Shading style */
	shading?: "none" | "subtle";
	/** Stacking order used to resolve Z-fighting when regions overlap (0 = bottom) */
	zIndex?: number;
}

export interface SignProps {
	/** Label text to display on the sign */
	label: string;
	/** Background color of the sign panel */
	color?: string;
	/** Text color */
	labelColor?: string;
	/** Font size for the label */
	fontSize?: number;
	/** Padding around the label inside the sign */
	margin?: number;
}

/**
 * A slim rectangular sign panel that displays a text label with a small margin.
 */
export function Sign({
	label,
	color = "#2d2421",
	labelColor = "#f5f0eb",
	fontSize = 0.5,
	margin = 0.04,
}: SignProps) {
	const signWidth = label.length * fontSize * 0.52 + margin * 2;
	const signHeight = fontSize + margin * 2;
	const signDepth = 0.04;

	return (
		<group>
			<mesh>
				<boxGeometry args={[signWidth, signHeight, signDepth]} />
				<meshStandardMaterial color={color} metalness={0.1} roughness={0.7} />
			</mesh>
			<Text
				position={[0, 0, signDepth / 2 + 0.001]}
				fontSize={fontSize}
				color={labelColor}
				anchorX="center"
				anchorY="middle"
			>
				{label}
			</Text>
		</group>
	);
}

/**
 * 3D Region Component for React Three Fiber
 * Renders a flat plane defined by four corner points
 * Perfect for defining areas or zones in 3D scenes
 */
export function Region({
	p1,
	p2,
	p3,
	p4,
	color = "#d4a574",
	label,
	labelColor = "#2d2421",
	labelSize = 0.156,
	opacity = 1,
	shading = "subtle",
	zIndex = 0,
}: RegionProps) {
	const groupRef = useRef<Group>(null);

	// Normalize positions to 3D coordinates
	const normalizePos = (
		p: [number] | [number, number] | [number, number, number],
	): THREE.Vector3 => {
		return new THREE.Vector3(p[0], p[2] ? p[1] : 0, p[2] ? p[2] : p[1]);
	};

	const pos1 = normalizePos(p1);
	const pos2 = normalizePos(p2);
	const pos3 = normalizePos(p3);
	const pos4 = normalizePos(p4);

	// Calculate plane center and normal
	const center = pos1
		.clone()
		.add(pos2)
		.add(pos3)
		.add(pos4)
		.multiplyScalar(0.25);

	// Calculate edges to determine plane orientation
	const edge1 = pos2.clone().sub(pos1);
	const edge2 = pos4.clone().sub(pos1);

	// Calculate normal vector
	const normal = edge1.clone().cross(edge2).normalize();

	// Create quaternion to orient plane along normal
	const quaternion = new THREE.Quaternion();
	const zAxis = new THREE.Vector3(0, 0, 1);
	quaternion.setFromUnitVectors(zAxis, normal);

	// Calculate plane dimensions
	const width = edge1.length();
	const height = edge2.length();

	// Build a plane geometry and (optionally) apply subtle vertex-color shading
	const planeGeometry = useMemo(() => {
		const geo = new THREE.PlaneGeometry(width, height, 1, 1);
		if (shading === "subtle") {
			// Compute per-vertex colors based on base color and vertex position
			const base = new THREE.Color(color);
			const posAttr = geo.getAttribute("position");
			const count = posAttr.count;
			const colors = new Float32Array(count * 3);

			// Determine half extents for normalization
			let maxX = 0;
			let maxY = 0;
			for (let i = 0; i < count; i++) {
				const x = posAttr.getX(i);
				const y = posAttr.getY(i);
				maxX = Math.max(maxX, Math.abs(x));
				maxY = Math.max(maxY, Math.abs(y));
			}

			for (let i = 0; i < count; i++) {
				const x = posAttr.getX(i);
				const y = posAttr.getY(i);
				const nx = maxX > 0 ? x / maxX : 0;
				const ny = maxY > 0 ? y / maxY : 0;
				// Subtle gradient factor: range ~[-0.07, 0.07]
				const f = Math.max(-0.07, Math.min(0.07, 0.05 * nx + 0.05 * ny));
				const shaded = base.clone().multiplyScalar(1 + f);
				colors[i * 3 + 0] = shaded.r;
				colors[i * 3 + 1] = shaded.g;
				colors[i * 3 + 2] = shaded.b;
			}

			geo.setAttribute("color", new THREE.Float32BufferAttribute(colors, 3));
		}
		return geo;
	}, [width, height, color, shading]);

	const groundTexture = useGroundTexture(color, width, height);

	// Block-face label: upright at the front edge, facing outward from the region.
	// outward = direction from center toward the front edge (along edge2)
	// planeUp = direction "up" from the plane surface (opposite to normal)
	const outward = edge2.clone().normalize();
	const planeUp = normal.clone().negate();

	// Build a basis where +Z faces outward, +Y is planeUp, so the text
	// stands upright and faces away from the region (like a block face).
	const labelRight = new THREE.Vector3()
		.crossVectors(planeUp, outward)
		.normalize();
	const labelUp = new THREE.Vector3()
		.crossVectors(outward, labelRight)
		.normalize();
	const blockFaceMatrix = new THREE.Matrix4().makeBasis(
		labelRight,
		labelUp,
		outward,
	);
	const blockFaceQuaternion = new THREE.Quaternion().setFromRotationMatrix(
		blockFaceMatrix,
	);

	// Position at the front edge, elevated above the surface.
	const labelLocalPos = outward
		.clone()
		.multiplyScalar(height / 2)
		.add(planeUp.clone().multiplyScalar(labelSize * 1.4));

	return (
		<group
			ref={groupRef}
			position={center.toArray() as [number, number, number]}
		>
			{/* 3D plane with optional subtle shading */}
			{/* Use white when vertex colors or texture carry the color, to avoid double-multiply */}
			<mesh
				quaternion={quaternion}
				geometry={planeGeometry}
				receiveShadow
				renderOrder={zIndex}
			>
				<meshStandardMaterial
					color={shading === "subtle" || groundTexture ? "#ffffff" : color}
					map={groundTexture ?? undefined}
					side={THREE.DoubleSide}
					transparent={opacity < 1}
					opacity={opacity}
					metalness={0.2}
					roughness={0.82}
					vertexColors={shading === "subtle"}
					polygonOffset
					polygonOffsetFactor={-1 - zIndex}
					polygonOffsetUnits={-1 - zIndex * 4}
				/>
			</mesh>
			{label && (
				<group
					quaternion={blockFaceQuaternion}
					position={labelLocalPos.toArray() as [number, number, number]}
				>
					<Sign
						label={label}
						color={color}
						labelColor={labelColor}
						fontSize={labelSize}
					/>
				</group>
			)}
		</group>
	);
}
