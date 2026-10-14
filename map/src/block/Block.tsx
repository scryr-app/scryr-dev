import { useEffect, useState } from "react";
import { cameraStore } from "@/camera";
import type { Block as GraphqlBlock } from "@/graphql/generated";
import { currentTheme } from "@/theme/theme";
import { BLOCK_DIMENSIONS } from "./dimensions";
import { TopLabel } from "./TopLabel";
import { Walls } from "./Walls";
import { ZoomButton } from "./ZoomButton";

export interface BlockProps {
	position?: [number, number, number];
	color?: string;
	name?: string;
	icon?: string;
	fontColor?: string;
	width?: number;
	height?: number;
	depth?: number;
	blockData?: GraphqlBlock;
}

/** Architecture block with a label and camera focus control. */
export function Block({
	position = [0, 0, 0],
	color = currentTheme.surface,
	name = "",
	icon = "",
	fontColor = currentTheme.fontColor,
	width = BLOCK_DIMENSIONS.width,
	height = BLOCK_DIMENSIONS.height,
	depth = BLOCK_DIMENSIONS.depth,
	blockData,
}: BlockProps) {
	const hw = width / 2;
	const hh = height / 2;
	const hd = depth / 2;
	const [isHovered, setIsHovered] = useState(false);
	const blockFocusId = blockData?.name ?? name;
	useEffect(() => {
		if (!blockFocusId) {
			return;
		}

		return cameraStore.registerBlock({
			id: blockFocusId,
			position,
			width,
			height,
			depth,
		});
	}, [blockFocusId, position, width, height, depth]);

	return (
		<group
			position={position as [number, number, number]}
			onPointerOver={(event) => {
				event.stopPropagation();
				setIsHovered(true);
			}}
			onPointerOut={() => setIsHovered(false)}
		>
			{/* Top label with icon and name */}
			<TopLabel
				icon={icon}
				name={name}
				fontColor={fontColor}
				hh={hh}
				width={width}
			/>

			{/* Outer and inner walls */}
			<Walls color={color} hw={hw} hh={hh} hd={hd} />

			<ZoomButton
				blockId={blockFocusId}
				isVisible={isHovered}
				blockPosition={position}
				blockWidth={width}
				blockHeight={height}
				blockDepth={depth}
			/>
		</group>
	);
}
