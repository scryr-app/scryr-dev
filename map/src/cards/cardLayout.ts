import type { ReactNode } from "react";
import type { CardSlotConfig } from "./CardSlot";

/** Keep toolbar indexes stable while assigning distinct offsets to visible cards. */
export function getCardLayout(
	cardConfigs: Array<{ components: ReactNode[] }>,
): CardSlotConfig[] {
	const count = cardConfigs.filter(
		(config) => config.components.length > 0,
	).length;
	let visibleIndex = 0;
	return cardConfigs.map((config) => ({
		...config,
		zOffset: config.components.length
			? (visibleIndex++ - (count - 1) / 2) * 0.1
			: 0,
	}));
}
