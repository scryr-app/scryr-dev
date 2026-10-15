// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { MapTray } from "./MapTray";
import { MapTrayProvider } from "./MapTrayContext";

vi.mock("@/theme/ThemeSwitcher", () => ({ ThemeOptions: () => null }));
afterEach(cleanup);
it.each([
	"Zoom In",
	"Zoom Out",
	"Pan Up",
	"Pan Down",
	"Pan Left",
	"Pan Right",
	"Rotate Up",
	"Rotate Down",
	"Rotate Left",
	"Rotate Right",
])("clears %s rollover when moving through the tray", (label) => {
	render(
		<MapTrayProvider>
			<MapTray isPyodideOpen={false} onTogglePyodide={vi.fn()} />
		</MapTrayProvider>,
	);
	const card = screen.getByRole("button", { name: "GitHub" });
	fireEvent.mouseEnter(card);
	expect(screen.getByText("GitHub")).toBeDefined();
	fireEvent.mouseLeave(card);
	const camera = screen.getByRole("button", { name: label });
	fireEvent.mouseEnter(camera);
	expect(screen.queryByText("GitHub")).toBeNull();
	expect(screen.getByText(label)).toBeDefined();
	fireEvent.mouseLeave(camera);
	fireEvent.mouseEnter(screen.getByRole("button", { name: "Select map view" }));
	expect(screen.queryByText(label)).toBeNull();
});
