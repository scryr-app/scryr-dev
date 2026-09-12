import { Text } from "@react-three/drei/core/Text";

const TOP_LABEL_FONT_SIZE = 0.28;
const TOP_LABEL_HORIZONTAL_PADDING = 0.2;

export interface TopLabelProps {
	/** Icon emoji to display before the name */
	icon: string;
	/** Name/title of the block */
	name: string;
	/** Text color */
	fontColor: string;
	/** Half-height of the block for positioning */
	hh: number;
	/** Width of the block that the label must fit within */
	width: number;
}

/**
 * TopLabel component renders the icon and name on top of a Block.
 * Positioned just above the block's top face, rotated to lie flat.
 */
export function TopLabel({ icon, name, fontColor, hh, width }: TopLabelProps) {
	const label = `${icon}${name}`.replace(/\s+/g, " ").trim();
	const maxLabelWidth = Math.max(0, width - TOP_LABEL_HORIZONTAL_PADDING);

	return (
		<Text
			position={[0, hh + 0.01, 0]}
			rotation={[-Math.PI / 2, 0, 0]}
			fontSize={TOP_LABEL_FONT_SIZE}
			color={fontColor}
			anchorX="center"
			anchorY="middle"
			whiteSpace="nowrap"
			onSync={(text) => {
				const [left, , right] = text.textRenderInfo.blockBounds;
				const renderedWidth = right - left;
				const scale =
					renderedWidth > 0 ? Math.min(1, maxLabelWidth / renderedWidth) : 1;
				text.scale.setScalar(scale);
			}}
		>
			{label}
		</Text>
	);
}
