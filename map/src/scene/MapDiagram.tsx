import { Environment } from "@react-three/drei/core/Environment";
import { Lightformer } from "@react-three/drei/core/Lightformer";
import { OrbitControls } from "@react-three/drei/core/OrbitControls";
import { PerspectiveCamera } from "@react-three/drei/core/PerspectiveCamera";
import { Canvas as DiagramSurface, useFrame } from "@react-three/fiber";
import { useRef } from "react";
import type {
	DirectionalLight as DirectionalLightType,
	PerspectiveCamera as PerspectiveCameraType,
} from "three";
import type { OrbitControls as OrbitControlsType } from "three-stdlib";
import { cameraStore } from "@/camera";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { currentTheme } from "@/theme/theme";
import { MapDisplay } from "./MapDisplay";

function MovingThemeLight() {
	const light = useRef<DirectionalLightType>(null);
	const motion = currentTheme.appearance.lighting.motion;
	useFrame(({ clock }) => {
		if (!light.current || !motion) return;
		const elapsed = clock.getElapsedTime();
		const speed = motion === "orbit" ? 0.075 : 0.18;
		const radius = motion === "orbit" ? 13 : 8;
		light.current.position.set(
			Math.cos(elapsed * speed) * radius,
			motion === "orbit" ? 12 : 15 + Math.sin(elapsed * 0.31) * 2,
			Math.sin(elapsed * speed) * radius,
		);
		light.current.intensity =
			motion === "caustic" ? 0.45 + Math.sin(elapsed * 0.7) * 0.12 : 0.25;
	});
	if (!motion) return null;
	return (
		<directionalLight
			ref={light}
			color={motion === "orbit" ? "#e5c47b" : "#8ee8d8"}
			intensity={0.28}
		/>
	);
}

export function MapDiagram() {
	const { lighting, view } = currentTheme.appearance;
	const cameraRef = useRef<PerspectiveCameraType>(null);

	const handleControlsMount = (controls: OrbitControlsType | null) => {
		cameraStore.controls = controls;
	};

	return (
		<ErrorBoundary
			name="Diagram"
			fallback={
				<div className="absolute inset-0 flex items-center justify-center text-slate-100">
					Map render failed.
				</div>
			}
		>
			<DiagramSurface
				shadows="percentage"
				gl={{
					preserveDrawingBuffer: true,
					localClippingEnabled: true,
				}}
			>
				<PerspectiveCamera
					ref={cameraRef}
					makeDefault
					fov={view.fov}
					position={view.position}
					near={0.1}
					far={1000}
				/>
				<ambientLight intensity={lighting.ambient} />
				<hemisphereLight
					args={[
						lighting.hemisphere.sky,
						lighting.hemisphere.ground,
						lighting.hemisphere.intensity,
					]}
				/>
				{lighting.fill.map((light) => (
					<directionalLight key={light.position.join(",")} {...light} />
				))}
				{lighting.reflections.length > 0 && (
					<Environment resolution={128} frames={1}>
						{lighting.reflections.map((light) => (
							<Lightformer
								key={light.position.join(",")}
								{...light}
								target={[0, 0, 0]}
							/>
						))}
					</Environment>
				)}
				<directionalLight
					castShadow
					{...lighting.key}
					shadow-mapSize-width={2048}
					shadow-mapSize-height={2048}
					shadow-bias={-0.00015}
					shadow-radius={3}
					shadow-camera-near={1}
					shadow-camera-far={80}
					shadow-camera-left={-30}
					shadow-camera-right={30}
					shadow-camera-top={30}
					shadow-camera-bottom={-30}
				/>
				<MovingThemeLight />
				<MapDisplay />
				<OrbitControls
					ref={handleControlsMount}
					target={[0, 0, 0]}
					maxPolarAngle={Math.PI / 2 - Math.PI / 18}
					minPolarAngle={0}
				/>
			</DiagramSurface>
		</ErrorBoundary>
	);
}
