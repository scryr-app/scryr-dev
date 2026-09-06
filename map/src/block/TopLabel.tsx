import { Text } from "@react-three/drei/core/Text";

const TOP_LABEL_FONT_SIZE = 0.28;

export interface TopLabelProps {
	/** Icon emoji to display before the name */
	icon: string;
	/** Name/title of the block */
	name: string;
	/** Text color */
	fontColor: string;
	/** Half-height of the block for positioning */
	hh: number;
}

/**
 * TopLabel component renders the icon and name on top of a Block.
 * Positioned just above the block's top face, rotated to lie flat.
 */
export function TopLabel({ icon, name, fontColor, hh }: TopLabelProps) {
	return (
		<Text
			position={[0, hh + 0.01, 0]}
			rotation={[-Math.PI / 2, 0, 0]}
			fontSize={TOP_LABEL_FONT_SIZE}
			color={fontColor}
			anchorX="center"
			anchorY="middle"
		>
			{icon}
			{name}
		</Text>
	);
}
