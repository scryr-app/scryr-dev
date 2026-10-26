import { useFrame, useThree } from "@react-three/fiber";
import { createContext, type ReactNode, useContext, useEffect, useMemo } from "react";
import { type Group, type Object3D } from "three";
import type { OrbitControls } from "three-stdlib";
import { useCardFocus } from "./CardFocusContext";

const FOREGROUND_LAYER = 31;
export const CARD_HTML_Z_RANGE: [number, number] = [2000000, 0];

/** The actual card objects change render layers; no clones or extra canvas. */
export function createCardForeground() {
	const cards = new Map<Group, Map<Object3D, number>>();
	return {
		cards,
		activate(group: Group) {
			let masks = cards.get(group);
			if (!masks) {
				masks = new Map();
				cards.set(group, masks);
			}
			const saved = masks;
			group.traverse((object) => {
				if (!saved.has(object)) saved.set(object, object.layers.mask);
				object.layers.set(FOREGROUND_LAYER);
			});
		},
		release(group: Group) {
			cards.get(group)?.forEach((mask, object) => {
				object.layers.mask = mask;
			});
			cards.delete(group);
		},
		contains(object: Object3D) {
			let current: Object3D | null = object;
			while (current) {
				if (cards.has(current as Group)) return true;
				current = current.parent;
			}
			return false;
		},
	};
}
const ForegroundContext = createContext<ReturnType<typeof createCardForeground> | null>(null);
export function useCardForeground() {
	const foreground = useContext(ForegroundContext);
	if (!foreground) throw new Error("CardSlot must be inside CardFocusScene");
	return foreground;
}

export function CardFocusScene({ children }: { children: ReactNode }) {
	const foreground = useMemo(createCardForeground, []);
	const { focusedIds } = useCardFocus();
	const { get, setEvents, raycaster } = useThree();
	useEffect(() => {
		const filter = get().events.filter;
		const mask = raycaster.layers.mask;
		raycaster.layers.enable(FOREGROUND_LAYER);
		setEvents({
			filter: (hits, state) => {
				const allowed = foreground.cards.size
					? hits.filter((hit) => foreground.contains(hit.object))
					: hits;
				return filter ? filter(allowed, state) : allowed;
			},
		});
		return () => {
			setEvents({ filter });
			raycaster.layers.mask = mask;
		};
	}, [foreground, get, setEvents, raycaster]);

	// Freeze navigation, not rendering: card motion and the background remain live.
	useEffect(() => {
		if (!focusedIds.length) return;
		const controls = get().controls as OrbitControls | null;
		if (!controls) return;
		const position = controls.object.position.clone();
		const target = controls.target.clone();
		const damping = controls.enableDamping;
		controls.enableDamping = false;
		controls.update();
		controls.object.position.copy(position);
		controls.target.copy(target);
		controls.update();
		controls.enableDamping = damping;
	}, [focusedIds.length > 0, get]);

	useFrame(({ gl, scene, camera, controls }) => {
		// Html's blending masks reveal DOM below this transparent canvas. Keep the
		// shared surface stable even when a loading/error Html mounts or unmounts.
		gl.domElement.style.zIndex = String(CARD_HTML_Z_RANGE[0] / 2);
		gl.domElement.style.position = "absolute";
		gl.domElement.style.pointerEvents = "none";
		if (controls)
			(controls as OrbitControls).enabled = !focusedIds.length && !foreground.cards.size;
		const autoClear = gl.autoClear;
		const mask = camera.layers.mask;
		gl.autoClear = true;
		camera.layers.disable(FOREGROUND_LAYER);
		gl.render(scene, camera);
		if (foreground.cards.size) {
			// Render the same foreground meshes after the diagram so nearby buildings
			// cannot obscure their faces. Clearing depth leaves the diagram visible.
			const background = scene.background;
			const shadowUpdate = gl.shadowMap.autoUpdate;
			const lightMasks = new Map<Object3D, number>();
			scene.traverse((object) => {
				if ("isLight" in object && object.isLight) {
					lightMasks.set(object, object.layers.mask);
					object.layers.enable(FOREGROUND_LAYER);
				}
			});
			gl.autoClear = false;
			gl.shadowMap.autoUpdate = false;
			gl.clearDepth();
			scene.background = null;
			camera.layers.set(FOREGROUND_LAYER);
			gl.render(scene, camera);
			scene.background = background;
			gl.shadowMap.autoUpdate = shadowUpdate;
			lightMasks.forEach((lightMask, light) => {
				light.layers.mask = lightMask;
			});
		}
		camera.layers.mask = mask;
		gl.autoClear = autoClear;
	}, 1);
	return <ForegroundContext.Provider value={foreground}>{children}</ForegroundContext.Provider>;
}
