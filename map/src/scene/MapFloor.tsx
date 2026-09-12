import { useFrame } from "@react-three/fiber";
import { useEffect, useMemo } from "react";
import { currentTheme } from "@/theme/theme";
import { createFloorTexture } from "./floorTexture";

export function MapFloor() {
	const floor = currentTheme.appearance.floor;
	const texture = useMemo(
		() => createFloorTexture(...floor.tiles, floor.pattern),
		[floor],
	);
	useEffect(() => () => texture?.dispose(), [texture]);
	useFrame(({ clock }) => {
		if (!texture) return;
		const elapsed = clock.getElapsedTime();
		if (floor.pattern === "stars") {
			texture.center.set(0.5, 0.5);
			texture.rotation = elapsed * 0.0025;
		} else if (floor.pattern === "seabed") {
			texture.offset.x = Math.sin(elapsed * 0.16) * 0.012;
			texture.offset.y = Math.cos(elapsed * 0.12) * 0.01;
		}
	});
	return (
		<>
			<mesh rotation-x={-Math.PI / 2} position={[0, -0.025, 0]} receiveShadow>
				<planeGeometry args={[52, 52]} />
				<meshStandardMaterial
					map={texture}
					color="#ffffff"
					transparent
					opacity={floor.opacity}
					roughness={floor.roughness}
					metalness={floor.metalness}
				/>
			</mesh>
			<gridHelper
				args={[50, 50, floor.gridMajor, floor.gridMinor]}
				onUpdate={(helper) => {
					helper.material.transparent = true;
					helper.material.opacity = floor.gridOpacity;
				}}
			/>
		</>
	);
}
