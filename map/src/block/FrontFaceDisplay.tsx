// FrontFaceDisplay intentionally renders nothing — service metadata
// has been moved to OverviewCard in the card slot system.

/** Kept for compatibility; the front face of a block is now bare. */
export interface FrontFaceDisplayProps {
	color?: string;
	width?: number;
	hh?: number;
	hd?: number;
}

/** No-op component. The front face of the block is intentionally empty. */
export function FrontFaceDisplay(_props: FrontFaceDisplayProps) {
	return null;
}
