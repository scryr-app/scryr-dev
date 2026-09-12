// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { ThemeOptions } from "./ThemeSwitcher";
import { getTheme, resolveThemeId, setThemePreset, useTheme } from "./theme";

beforeEach(() => {
	localStorage.clear();
	setThemePreset("IndustrialForest");
});
afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

function Preview() {
	const theme = useTheme();
	const [draft, setDraft] = useState("unsaved source");
	return (
		<>
			<ThemeOptions />
			<output aria-label="Appearance">
				{theme.diagramMode} / {theme.appearance.textures.wall} /{" "}
				{theme.appearance.connections.luminous ? "glow" : "matte"}
			</output>
			<input
				aria-label="Draft"
				value={draft}
				onChange={(event) => setDraft(event.target.value)}
			/>
		</>
	);
}

it("switches the whole appearance and persists it without remounting the draft", () => {
	render(<Preview />);
	const draft = screen.getByRole("textbox", { name: "Draft" });
	fireEvent.change(draft, { target: { value: "keep my changes" } });
	localStorage.setItem("diagramMode", "light");
	fireEvent.click(screen.getByRole("button", { name: /Luminous Crystal/ }));
	expect(screen.getByLabelText("Appearance").textContent).toBe(
		"dark / crystal / glow",
	);
	expect(
		screen
			.getByRole("button", { name: /Luminous Crystal/ })
			.getAttribute("aria-pressed"),
	).toBe("true");
	expect(localStorage.getItem("selectedTheme")).toBe("LightningNeon");
	expect(localStorage.getItem("diagramMode")).toBeNull();
	expect(screen.getByRole("textbox", { name: "Draft" })).toBe(draft);
	expect((draft as HTMLInputElement).value).toBe("keep my changes");
	fireEvent.click(screen.getByRole("button", { name: /Scholarly Grimoire/ }));
	expect(screen.getByLabelText("Appearance").textContent).toBe(
		"dark / leather / matte",
	);
	expect(localStorage.getItem("selectedTheme")).toBe("ScholarsGrimoire");
	expect(getTheme().cardTextColor).toBe("#fff5df");
	expect((draft as HTMLInputElement).value).toBe("keep my changes");
	fireEvent.click(screen.getByRole("button", { name: /Industrial Forest/ }));
	expect(screen.getByLabelText("Appearance").textContent).toBe(
		"light / ribbed / matte",
	);
	expect(localStorage.getItem("selectedTheme")).toBe("IndustrialForest");
	expect(screen.queryByRole("switch")).toBeNull();
});

it.each([
	["LightningNeon", "light", "LightningNeon"],
	["ScholarsGrimoire", "light", "ScholarsGrimoire"],
	["CelestialObservatory", "light", "CelestialObservatory"],
	["PorcelainReverie", "dark", "PorcelainReverie"],
	["SunkenSanctuary", "light", "SunkenSanctuary"],
	["VelvetOracle", "dark", "VelvetOracle"],
	["IndustrialForest", "dark", "IndustrialForest"],
	["AutumnOffice", "dark", "LightningNeon"],
	["ForestFactory", "light", "IndustrialForest"],
	["SteelBlue", null, "LightningNeon"],
	["unknown", null, "IndustrialForest"],
	["toString", null, "IndustrialForest"],
	[null, null, "IndustrialForest"],
])(
	"resolves saved theme %s with legacy brightness %s",
	(selected, mode, expected) => {
		expect(resolveThemeId(selected, mode)).toBe(expected);
	},
);

it.each([
	["CelestialObservatory", "dark", "celestial", "orbit"],
	["PorcelainReverie", "light", "porcelain", undefined],
	["SunkenSanctuary", "dark", "sunkenStone", "caustic"],
	["VelvetOracle", "light", "velvet", undefined],
] as const)(
	"defines the complete %s scene and editor theme",
	(id, mode, wall, motion) => {
		setThemePreset(id);
		const theme = getTheme();
		expect(theme.diagramMode).toBe(mode);
		expect(theme.appearance.textures.wall).toBe(wall);
		expect(theme.appearance.lighting.motion).toBe(motion);
		expect(theme.console.background).not.toBe(theme.background);
	},
);

it("still switches when persistent storage is unavailable", () => {
	vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
		throw new Error("disabled");
	});
	setThemePreset("LightningNeon");
	expect(getTheme().diagramMode).toBe("dark");
	expect(getTheme().appearance.lighting.reflections.length).toBeGreaterThan(0);
});
