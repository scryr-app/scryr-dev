import { useFrame, useThree } from "@react-three/fiber";
import { useEffect, useMemo, useRef } from "react";
import { Group, PerspectiveCamera, Quaternion, Vector3 } from "three";
import { useCardFocus } from "./CardFocusContext";
import { useCardForeground } from "./CardFocusScene";
import { getCardFocusLayout } from "./cardFocusLayout";

/** Moves the card in its existing parent, preserving both mesh identity and content. */
export function useCardFocusAnimation(id: string, width: number, height: number, zOffset: number) {
	const ref = useRef<Group>(null);
	const { focusedIds, expandedId } = useCardFocus();
	const foreground = useCardForeground();
	const size = useThree((state) => state.size);
	const visibleIds = expandedId ? [expandedId] : focusedIds;
	const index = visibleIds.indexOf(id);
	const layout = useMemo(
		() => getCardFocusLayout(size.width, size.height, visibleIds.length, width / height),
		[size.width, size.height, visibleIds.length, width, height],
	);
	const moving = useRef(false);
	const vectors = useMemo(
		() => ({
			position: new Vector3(),
			scale: new Vector3(1, 1, 1),
			rotation: new Quaternion(),
			cameraPosition: new Vector3(),
			cameraRotation: new Quaternion(),
			parentRotation: new Quaternion(),
			parentScale: new Vector3(),
		}),
		[],
	);
	useEffect(() => {
		const group = ref.current;
		return () => {
			if (group) foreground.release(group);
		};
	}, [foreground]);

	useFrame(({ camera }, delta) => {
		const group = ref.current;
		if (!group?.parent || !(camera instanceof PerspectiveCamera)) return;
		const rect = layout[index];
		const {
			position,
			scale,
			rotation,
			cameraRotation,
			cameraPosition,
			parentRotation,
			parentScale,
		} = vectors;
		if (rect) {
			moving.current = true;
			// At this distance cards are physically enlarged as well as brought forward.
			const distance = Math.max(8, camera.near * 20);
			const worldPerPixel =
				(2 * distance * Math.tan((camera.getEffectiveFOV() * Math.PI) / 360)) /
				Math.max(1, size.height);
			camera.getWorldQuaternion(cameraRotation);
			position
				.set(rect.x * worldPerPixel, rect.y * worldPerPixel, -distance)
				.applyQuaternion(cameraRotation)
				.add(camera.getWorldPosition(cameraPosition));
			group.parent.updateWorldMatrix(true, false);
			group.parent.worldToLocal(position);
			group.parent.getWorldQuaternion(parentRotation);
			rotation.copy(parentRotation).invert().multiply(cameraRotation);
			group.parent.getWorldScale(parentScale);
			const factor = ((rect.width * worldPerPixel) / width) * 0.98;
			scale.set(factor / parentScale.x, factor / parentScale.y, factor / parentScale.z);
		} else {
			if (!moving.current) return;
			position.set(0, 0, zOffset);
			rotation.identity();
			scale.set(1, 1, 1);
		}
		const alpha = 1 - Math.exp(-8 * Math.min(delta, 0.1));
		group.position.lerp(position, alpha);
		group.quaternion.slerp(rotation, alpha);
		group.scale.lerp(scale, alpha);
		foreground.activate(group);
		if (
			!rect &&
			group.position.distanceToSquared(position) < 0.000001 &&
			group.scale.distanceToSquared(scale) < 0.000001 &&
			group.quaternion.angleTo(rotation) < 0.001
		) {
			group.position.copy(position);
			group.quaternion.copy(rotation);
			group.scale.copy(scale);
			moving.current = false;
			foreground.release(group);
		}
	});
	return ref;
}
