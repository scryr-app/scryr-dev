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
import { setDiagramMode } from "@/theme/theme";
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
	PythonCodeEditor: ({ theme }: { theme: string }) => (
		<div data-testid="code" data-theme={theme} />
	),
}));
beforeEach(() => {
	vi.clearAllMocks();
	setDiagramMode("light");
});
afterEach(cleanup);

function consoleView() {
	return render(
		<MapTrayProvider>
			<PyodideConsole
				isOpen
				panelWidth={480}
				onOpenChange={vi.fn()}
				onPanelWidthChange={vi.fn()}
			/>
		</MapTrayProvider>,
	);
}
it("labels the action Save and Run and groups source controls with the bottom status", () => {
	consoleView();
	fireEvent.click(
		screen.getByRole("button", { name: "Save and Run" }),
	);
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
it("inherits map mode changes while keeping tray chrome separate from the code area", () => {
	consoleView();
	const code = screen.getByTestId("code");
	const panel = screen.getByRole("region", { name: "Manifest source editor" });
	expect(code.getAttribute("data-theme")).toBe("light");
	expect(code.parentElement?.classList.contains("bg-white")).toBe(true);
	for (const token of ["bg-black/40", "border-white/15", "backdrop-blur-md"])
		expect(panel.classList.contains(token)).toBe(true);
	act(() => setDiagramMode("dark"));
	expect(code.getAttribute("data-theme")).toBe("dark");
	expect(code.parentElement?.classList.contains("bg-[#1e1e1e]")).toBe(true);
	expect(panel.classList.contains("bg-black/40")).toBe(true);
});
