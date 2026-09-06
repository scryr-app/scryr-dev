export function isScryrLocalAuthMode(authMode: string | undefined): boolean {
	return authMode === "local";
}

export const scryrAuthMode = import.meta.env.VITE_SCRYR_AUTH_MODE ?? "local";
export const isLocalAuthMode = isScryrLocalAuthMode(scryrAuthMode);
