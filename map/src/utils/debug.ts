/**
 * True when VITE_DEBUG=true is set in the environment (for example via
 * untracked `mise.local.toml`).
 *
 * Vite strips dead code inside blocks gated on this flag at build time,
 * so debug-only components will not appear in production bundles unless
 * you explicitly opt in.
 *
 * @example
 * ```tsx
 * import { isDebug } from "@/utils";
 * {isDebug && <DebugPanel />}
 * ```
 *
 * To enable, add to `mise.local.toml`:
 *   [env]
 *   VITE_DEBUG="true"
 */
export const isDebug: boolean = import.meta.env.VITE_DEBUG === "true";
