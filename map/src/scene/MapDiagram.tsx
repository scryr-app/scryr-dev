import { OrbitControls } from "@react-three/drei/core/OrbitControls";
import { PerspectiveCamera } from "@react-three/drei/core/PerspectiveCamera";
import { Canvas as DiagramSurface } from "@react-three/fiber";
import { useRef } from "react";
import type { PerspectiveCamera as PerspectiveCameraType } from "three";
import type { OrbitControls as OrbitControlsType } from "three-stdlib";
import { cameraStore } from "@/camera";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { MapDisplay } from "./MapDisplay";

export function MapDiagram() {
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
					fov={40}
					position={[8, 6, 8]}
					near={0.1}
					far={1000}
				/>
				<ambientLight intensity={0.4} />
				<directionalLight
					castShadow
					position={[12, 18, 10]}
					intensity={1.1}
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
