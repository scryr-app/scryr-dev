import { useState } from "react";
import { currentTheme } from "@/theme/theme";
import {
	createDarkFloorCheckerTexture,
	createLightFloorCheckerTexture,
} from "./floorTexture";

export function MapFloor() {
	const [lightFloorTexture] = useState(() => createLightFloorCheckerTexture());
	const [darkFloorTexture] = useState(() => createDarkFloorCheckerTexture());
	const isDarkDiagram = currentTheme.isDarkDiagram;
	const floorTexture = isDarkDiagram ? darkFloorTexture : lightFloorTexture;

	return (
		<>
			<mesh rotation-x={-Math.PI / 2} position={[0, -0.025, 0]} receiveShadow>
				<planeGeometry args={[52, 52]} />
				<meshStandardMaterial
					map={floorTexture}
					color="#ffffff"
					transparent
					opacity={isDarkDiagram ? 0.52 : 0.42}
					roughness={1}
					metalness={0}
				/>
			</mesh>
			<gridHelper
				args={[
					50,
					50,
					isDarkDiagram ? "#cbd5e1" : "#dbeafe",
					isDarkDiagram ? "#475569" : "#f8fafc",
				]}
				onUpdate={(helper) => {
					helper.material.transparent = true;
					helper.material.opacity = isDarkDiagram ? 0.24 : 0.16;
				}}
			/>
		</>
	);
}
