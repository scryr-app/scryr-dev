// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vite-plus/test";
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
	const card = screen.getByRole("button", { name: "Diagrams" });
	fireEvent.mouseEnter(card);
	expect(screen.getByText("Diagrams")).toBeDefined();
	fireEvent.mouseLeave(card);
	const camera = screen.getByRole("button", { name: label });
	fireEvent.mouseEnter(camera);
	expect(screen.queryByText("Diagrams")).toBeNull();
	expect(screen.getByText(label)).toBeDefined();
	fireEvent.mouseLeave(camera);
	fireEvent.mouseEnter(screen.getByRole("button", { name: "Select map view" }));
	expect(screen.queryByText(label)).toBeNull();
});

it("offers only Info and Diagrams card categories", () => {
	render(
		<MapTrayProvider>
			<MapTray isPyodideOpen={false} onTogglePyodide={vi.fn()} />
		</MapTrayProvider>,
	);
	expect(screen.getByRole("button", { name: "Info" })).toBeDefined();
	const diagrams = screen.getByRole("button", { name: "Diagrams" });
	fireEvent.click(diagrams);
	expect(diagrams.getAttribute("aria-pressed")).toBe("true");
	for (const name of ["GitHub", "Metrics", "CI/CD", "Tests", "Dependencies", "Performance"]) {
		expect(screen.queryByRole("button", { name })).toBeNull();
	}
});
