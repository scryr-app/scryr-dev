import { describe, expect, it } from "vitest";
import { isLocalAuthMode, isScryrLocalAuthMode, scryrAuthMode } from "./env";

describe("Scryr app env", () => {
	it("defaults to local auth for no-cloud builds", () => {
		expect(scryrAuthMode).toBe("local");
		expect(isLocalAuthMode).toBe(true);
	});

	it("enables local auth only for the explicit local mode", () => {
		expect(isScryrLocalAuthMode("local")).toBe(true);
		expect(isScryrLocalAuthMode("clerk")).toBe(false);
		expect(isScryrLocalAuthMode(undefined)).toBe(false);
	});
});
