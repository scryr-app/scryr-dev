import { useThree } from "@react-three/fiber";
import { useEffect } from "react";
import { PerspectiveCamera } from "three";
import type { OrbitControls } from "three-stdlib";
import type { LayoutResult } from "./layout";
import { getLayoutOverview, type LayoutView } from "./layoutOverview";

/** Fit once per layout, leaving subsequent orbit, zoom and view switches alone. */
export function MapOverview({
	layout,
	view,
}: {
	layout: LayoutResult;
	view: LayoutView;
}) {
	const camera = useThree((state) => state.camera);
	const controls = useThree((state) => state.controls) as OrbitControls | null;
	useEffect(() => {
		if (!(camera instanceof PerspectiveCamera) || !controls) return;
		const overview = getLayoutOverview(layout, view);
		camera.position.copy(overview.camera.position);
		camera.far = overview.camera.far;
		camera.updateProjectionMatrix();
		controls.target.copy(overview.target);
		controls.update();
	}, [camera, controls, layout, view]);
	return null;
}
