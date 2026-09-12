import { useEffect, useMemo } from "react";
import * as THREE from "three";
import { currentTheme } from "@/theme/theme";

interface GeometryDimensions {
	hw: number; // half width
	hh: number; // half height
	hd: number; // half depth
}

/**
 * Creates a custom box geometry with 5 faces (no right face for card slots).
 */
export function useBlockGeometry({ hw, hh, hd }: GeometryDimensions) {
	const { flatFaces } = currentTheme.appearance.shapes;
	const geometry = useMemo(() => {
		const geo = new THREE.BufferGeometry();

		const vertices = [];
		const indices = [];

		// Define the 8 vertices of a box
		const verts = [
			[-hw, -hh, -hd], // 0: left-bottom-back
			[-hw, -hh, hd], // 1: left-bottom-front
			[-hw, hh, -hd], // 2: left-top-back
			[-hw, hh, hd], // 3: left-top-front
			[hw, -hh, -hd], // 4: right-bottom-back
			[hw, -hh, hd], // 5: right-bottom-front
			[hw, hh, -hd], // 6: right-top-back
			[hw, hh, hd], // 7: right-top-front
		];

		// Add all vertices
		for (const vert of verts) {
			vertices.push(...vert);
		}

		// Define faces (each face has 2 triangles = 6 indices)
		// We exclude the right face (would be indices [4,7,6,4,5,7])

		// Left face (facing negative X)
		indices.push(0, 3, 2, 0, 1, 3);
		// Top face (facing positive Y)
		indices.push(2, 3, 7, 2, 7, 6);
		// Bottom face (facing negative Y)
		indices.push(0, 1, 5, 0, 5, 4);
		// Front face (facing positive Z)
		indices.push(1, 5, 7, 1, 7, 3);
		// Back face (facing negative Z)
		indices.push(0, 6, 2, 0, 4, 6);

		geo.setIndex(indices);
		geo.setAttribute("position", new THREE.Float32BufferAttribute(vertices, 3));
		// Separate normals and UVs keep the polished faces crisp.
		geo.computeVertexNormals();
		const faces = geo.toNonIndexed();
		geo.dispose();
		const uvs = [];
		const positions = faces.getAttribute("position");
		for (let i = 0; i < positions.count; i++) {
			const face = Math.floor(i / 6);
			const x = positions.getX(i),
				y = positions.getY(i),
				z = positions.getZ(i);
			// Project each face in its own plane so adjacent triangles share UVs.
			const u = face === 0 ? (z + hd) / (2 * hd) : (x + hw) / (2 * hw);
			const v =
				face === 1 || face === 2 ? (z + hd) / (2 * hd) : (y + hh) / (2 * hh);
			uvs.push(u, v);
		}
		faces.setAttribute("uv", new THREE.Float32BufferAttribute(uvs, 2));
		if (flatFaces) faces.computeVertexNormals();
		return faces;
	}, [hd, hh, hw, flatFaces]);
	useEffect(() => () => geometry.dispose(), [geometry]);
	return geometry;
}

/**
 * Creates inner wall geometry with slight inset for visual depth.
 */
export function useInnerWallsGeometry({ hw, hh, hd }: GeometryDimensions) {
	const geometry = useMemo(() => {
		const geo = new THREE.BufferGeometry();
		const thickness = 0.05; // Wall thickness

		const vertices = [];
		const indices = [];

		// Inner vertices (slightly inset)
		const innerVerts = [
			[-hw + thickness, -hh + thickness, -hd + thickness], // 0: inner-left-bottom-back
			[-hw + thickness, -hh + thickness, hd - thickness], // 1: inner-left-bottom-front
			[-hw + thickness, hh - thickness, -hd + thickness], // 2: inner-left-top-back
			[-hw + thickness, hh - thickness, hd - thickness], // 3: inner-left-top-front
			[hw - thickness, -hh + thickness, -hd + thickness], // 4: inner-right-bottom-back
			[hw - thickness, -hh + thickness, hd - thickness], // 5: inner-right-bottom-front
			[hw - thickness, hh - thickness, -hd + thickness], // 6: inner-right-top-back
			[hw - thickness, hh - thickness, hd - thickness], // 7: inner-right-top-front
		];

		// Add inner vertices
		for (const vert of innerVerts) {
			vertices.push(...vert);
		}

		// Inner faces (reverse winding to face inward)
		// Left inner face
		indices.push(0, 2, 3, 0, 3, 1);
		// Top inner face
		indices.push(2, 6, 7, 2, 7, 3);
		// Bottom inner face
		indices.push(0, 5, 4, 0, 1, 5);
		// Front inner face
		indices.push(1, 3, 7, 1, 7, 5);
		// Back inner face
		indices.push(0, 4, 6, 0, 6, 2);

		geo.setIndex(indices);
		geo.setAttribute("position", new THREE.Float32BufferAttribute(vertices, 3));
		geo.computeVertexNormals();

		return geo;
	}, [hd, hh, hw]);
	useEffect(() => () => geometry.dispose(), [geometry]);
	return geometry;
}
