// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { MapToolbar } from "./MapToolbar";

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
])("clears %s rollover when moving through the toolbar", (label) => {
	render(<MapToolbar isPyodideOpen={false} onTogglePyodide={vi.fn()} />);
	const camera = screen.getByRole("button", { name: label });
	fireEvent.mouseEnter(camera);
	expect(screen.getByText(label)).toBeDefined();
	fireEvent.mouseLeave(camera);
	fireEvent.mouseEnter(screen.getByRole("button", { name: "Select map view" }));
	expect(screen.queryByText(label)).toBeNull();
});
it("keeps editor controls without exposing removed cards", () => {
	const toggle = vi.fn();
	render(<MapToolbar isPyodideOpen={false} onTogglePyodide={toggle} />);
	fireEvent.click(screen.getByRole("button", { name: "Show editor" }));
	expect(toggle).toHaveBeenCalledOnce();
	for (const name of [
		"Info",
		"GitHub",
		"Metrics",
		"CI/CD",
		"Tests",
		"Dependencies",
		"Performance",
		"Diagrams",
	]) {
		expect(screen.queryByRole("button", { name })).toBeNull();
	}
});
