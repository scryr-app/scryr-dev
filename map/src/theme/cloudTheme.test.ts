// @vitest-environment jsdom
import { expect, it, vi } from "vitest";

vi.mock("@/auth/env", () => ({ isLocalAuthMode: false }));

import { getTheme, resolveThemeId } from "./theme";

it("defaults cloud accounts to Luminous Crystal while preserving saved preferences", () => {
	expect(getTheme().id).toBe("LightningNeon");
	expect(resolveThemeId(null, null)).toBe("LightningNeon");
	expect(resolveThemeId("unknown", null)).toBe("LightningNeon");
	expect(resolveThemeId("IndustrialForest", null)).toBe("IndustrialForest");
	expect(resolveThemeId(null, "light")).toBe("IndustrialForest");
});
