import type { ReactNode } from "react";
import type { CardSlotConfig } from "./CardSlot";

/**
 * Calculate z-offset for cards based on total component count
 * Distributes cards evenly around the center (z=0)
 * Supports 0-5 total components
 */
export function getCardLayout(
	cardConfigs: Array<{ components: ReactNode[] }>,
): CardSlotConfig[] {
	// Calculate total component count
	const totalComponents = cardConfigs.reduce(
		(sum, config) => sum + config.components.length,
		0,
	);

	// Clamp between 0-7
	const count = Math.max(0, Math.min(7, totalComponents));

	// Define offset patterns based on total component count
	const offsetPatterns: Record<number, number[]> = {
		0: [],
		1: [0],
		2: [-0.1, 0.1],
		3: [0, 0.2, -0.2],
		4: [-0.15, -0.05, 0.05, 0.15],
		5: [-0.2, -0.1, 0, 0.1, 0.2],
		6: [-0.25, -0.15, -0.05, 0.05, 0.15, 0.25],
		7: [-0.3, -0.2, -0.1, 0, 0.1, 0.2, 0.3],
	};

	const offsets = offsetPatterns[count] || [];

	// Map offsets to card configs
	return cardConfigs.map((config, index) => ({
		...config,
		zOffset: offsets[index] ?? 0,
	}));
}
