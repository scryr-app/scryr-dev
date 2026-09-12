import { useMemo } from "react";
import * as THREE from "three";

const vertexShader = `
 varying vec2 frameUv;
 void main() {
   frameUv = uv;
   gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
 }
`;

const fragmentShader = `
 varying vec2 frameUv;
 uniform vec2 frameSize;
 uniform vec3 tint;
 uniform float strength;
 void main() {
   vec2 p = (frameUv - 0.5) * (frameSize + 0.32);
   vec2 q = abs(p) - frameSize * 0.5 + 0.035;
   float edge = abs(length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - 0.035);
   float core = exp(-edge * 180.0);
   float halo = exp(-edge * 26.0) * 0.28;
   gl_FragColor = vec4(tint, (core * 0.75 + halo) * strength);
   #include <colorspace_fragment>
 }
`;

/** A thin illuminated bevel with a soft, depth-tested halo. */
export function GlowFrame({
	width,
	height,
	color,
	strength = 0.65,
}: {
	width: number;
	height: number;
	color: string;
	strength?: number;
}) {
	const uniforms = useMemo(
		() => ({
			frameSize: { value: new THREE.Vector2(width, height) },
			tint: {
				value: new THREE.Color(color).lerp(new THREE.Color("#fff0d9"), 0.12),
			},
			strength: { value: strength },
		}),
		[width, height, color, strength],
	);
	return (
		<mesh raycast={() => {}}>
			<planeGeometry args={[width + 0.32, height + 0.32]} />
			<shaderMaterial
				uniforms={uniforms}
				vertexShader={vertexShader}
				fragmentShader={fragmentShader}
				transparent
				depthWrite={false}
				side={THREE.DoubleSide}
				blending={THREE.AdditiveBlending}
				toneMapped={false}
			/>
		</mesh>
	);
}
