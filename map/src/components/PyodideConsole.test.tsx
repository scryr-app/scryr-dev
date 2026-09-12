// @vitest-environment jsdom
import {
	act,
	cleanup,
	fireEvent,
	render,
	screen,
	within,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { MapTrayProvider } from "@/cards/MapTrayContext";
import { consoleTextColor } from "@/theme/console";
import { getTheme, setThemePreset, type Theme } from "@/theme/theme";
import { PyodideConsole } from "./PyodideConsole";

const editor = vi.hoisted(() => ({
	doc: {
		folderPath: "",
		entrypoint: "index.scry",
		writable: true,
		local: true,
	},
	code: "source",
	dirty: false,
	running: false,
	conflict: false,
	canCancel: false,
	status: "Ready",
	output: "",
	run: vi.fn(),
	reload: vi.fn(),
	cancel: vi.fn(),
	setCode: vi.fn(),
}));
vi.mock("@/graphql/useManifestEditor", () => ({
	useManifestEditor: () => editor,
}));
vi.mock("./PythonCodeEditor", () => ({
	PythonCodeEditor: ({ theme }: { theme: Theme }) => (
		<div data-testid="code" data-theme={theme.id} />
	),
}));
beforeEach(() => {
	vi.clearAllMocks();
	setThemePreset("IndustrialForest");
});
afterEach(cleanup);

function consoleView(onPanelWidthChange = vi.fn()) {
	return render(
		<MapTrayProvider>
			<PyodideConsole
				isOpen
				panelWidth={480}
				onOpenChange={vi.fn()}
				onPanelWidthChange={onPanelWidthChange}
			/>
		</MapTrayProvider>,
	);
}
it("labels the action Save and Run and groups source controls with the bottom status", () => {
	consoleView();
	fireEvent.click(screen.getByRole("button", { name: "Save and Run" }));
	expect(editor.run).toHaveBeenCalledOnce();
	const panel = screen.getByRole("region", { name: "Manifest source editor" });
	const footer = panel.querySelector("footer");
	expect(footer).not.toBeNull();
	if (!footer) throw new Error("Missing bottom toolbar");
	expect(
		within(footer).getByRole("checkbox", { name: "Follow selected block" }),
	).toBeDefined();
	fireEvent.click(
		within(footer).getByRole("button", { name: "Reload source" }),
	);
	expect(editor.reload).toHaveBeenCalledOnce();
	expect(within(footer).getByRole("status").textContent).toBe("Ready");
	expect(screen.queryByText(/Run saves/)).toBeNull();
	expect(screen.queryByRole("button", { name: /Use .* theme/ })).toBeNull();
});
it("updates the console palette with each theme without remounting the editor", () => {
	consoleView();
	const code = screen.getByTestId("code");
	const panel = screen.getByRole("region", { name: "Manifest source editor" });
	for (const id of [
		"IndustrialForest",
		"LightningNeon",
		"ScholarsGrimoire",
		"CelestialObservatory",
		"PorcelainReverie",
		"SunkenSanctuary",
		"VelvetOracle",
	] as const) {
		act(() => setThemePreset(id));
		const palette = getTheme().console;
		expect(screen.getByTestId("code")).toBe(code);
		expect(code.getAttribute("data-theme")).toBe(id);
		expect(panel.style.color).toBe(consoleTextColor(palette.text));
		const expected = document.createElement("div");
		expected.style.backgroundColor = palette.background;
		expect(code.parentElement?.style.backgroundColor).toBe(
			expected.style.backgroundColor,
		);
		expect(panel.classList.contains("backdrop-blur-md")).toBe(true);
	}
});

// jsdom does not implement pointer capture or pointer event coordinates.
function pointer(
	target: HTMLElement | Window,
	type: string,
	x: number,
	y: number,
) {
	fireEvent(
		target,
		Object.assign(
			new MouseEvent(type, {
				bubbles: true,
				button: 0,
				clientX: x,
				clientY: y,
			}),
			{ pointerId: 1 },
		),
	);
}

it("drags by the title bar, bounds movement, and ends dragging on cancellation", () => {
	const view = consoleView();
	const panel = screen.getByRole("region", { name: "Manifest source editor" });
	const bar = screen.getByTitle("Drag to move console");
	bar.setPointerCapture = vi.fn();
	vi.spyOn(panel, "getBoundingClientRect").mockReturnValue({
		width: 480,
		height: 500,
	} as DOMRect);
	const code = screen.getByTestId("code");
	pointer(bar, "pointerdown", 100, 90);
	pointer(window, "pointermove", 300, 130);
	expect(panel.style.left).toBe("216px");
	expect(panel.style.top).toBe("112px");
	expect(screen.getByTestId("code")).toBe(code);
	expect(document.body.style.cursor).toBe("grabbing");
	pointer(window, "pointermove", 5000, 5000);
	expect(panel.style.left).toBe(`${window.innerWidth - 488}px`);
	expect(panel.style.top).toBe(`${window.innerHeight - 508}px`);
	pointer(window, "pointermove", -5000, -5000);
	expect(panel.style.left).toBe("8px");
	expect(panel.style.top).toBe("8px");
	pointer(window, "pointercancel", 0, 0);
	pointer(window, "pointermove", 300, 130);
	expect(panel.style.left).toBe("8px");
	expect(document.body.style.cursor).toBe("");
	expect(document.body.style.userSelect).toBe("");
	pointer(bar, "pointerdown", 100, 90);
	view.unmount();
	expect(document.body.style.cursor).toBe("");
});

it("keeps title bar actions clickable and limits resizing at the moved right edge", () => {
	const resize = vi.fn();
	consoleView(resize);
	const panel = screen.getByRole("region", { name: "Manifest source editor" });
	const bar = screen.getByTitle("Drag to move console");
	bar.setPointerCapture = vi.fn();
	vi.spyOn(panel, "getBoundingClientRect").mockReturnValue({
		width: 480,
		height: 500,
	} as DOMRect);
	const run = screen.getByRole("button", { name: "Save and Run" });
	pointer(run, "pointerdown", 300, 90);
	pointer(window, "pointermove", 400, 120);
	pointer(window, "pointerup", 400, 120);
	fireEvent.click(run);
	expect(editor.run).toHaveBeenCalledOnce();
	expect(panel.style.left).toBe("16px");
	pointer(bar, "pointerdown", 100, 90);
	pointer(window, "pointermove", 300, 90);
	pointer(window, "pointerup", 300, 90);
	const handle = screen.getByRole("button", { name: "Resize terminal panel" });
	handle.setPointerCapture = vi.fn();
	pointer(handle, "pointerdown", 696, 200);
	pointer(window, "pointermove", 5000, 200);
	expect(resize).toHaveBeenLastCalledWith(window.innerWidth - 216 - 8);
	pointer(window, "pointerup", 5000, 200);
});
