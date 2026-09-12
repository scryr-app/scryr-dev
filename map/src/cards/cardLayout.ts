import type { ReactNode } from "react";
import type { CardSlotConfig } from "./CardSlot";

/** Distribute occupied category slots across the block's depth, without a card-count cap. */
export function getCardLayout(
	cardConfigs: Array<{
		categoryIndex?: number;
		id?: string;
		label?: string;
		components: ReactNode[];
	}>,
): CardSlotConfig[] {
	const occupied = cardConfigs.filter((c) => c.components.length > 0).length;
	let position = 0;
	return cardConfigs.map((config) => ({
		...config,
		zOffset:
			config.components.length === 0 || occupied <= 1
				? 0
				: -0.3 + (position++ / (occupied - 1)) * 0.6,
	}));
}
