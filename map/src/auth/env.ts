export function isScryrLocalAuthMode(authMode: string | undefined): boolean {
	return authMode === "local";
}

declare global {
	interface Window {
		__SCRYR_RUNTIME__?: { authMode: string; graphqlEndpoint: string };
	}
}
export const runtimeConfig =
	typeof window === "undefined" ? undefined : window.__SCRYR_RUNTIME__;
export const scryrAuthMode =
	runtimeConfig?.authMode ?? import.meta.env.VITE_SCRYR_AUTH_MODE ?? "local";
export const isLocalAuthMode = isScryrLocalAuthMode(scryrAuthMode);
