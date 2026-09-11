// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { ManifestDocument } from "./manifestDocument";

const mocks = vi.hoisted(() => ({
	read: vi.fn(),
	save: vi.fn(),
	run: vi.fn(),
	blocks: vi.fn(),
}));
vi.mock("./manifestDocument", () => ({
	documentKey: (id: string) => ["ManifestDocument", id],
	readDocument: mocks.read,
	saveDocument: mocks.save,
}));
vi.mock("@/pyodide/pyodideRuntime", () => ({ runPythonDocument: mocks.run }));
vi.mock("./client", () => ({ graphqlFetcher: () => mocks.blocks }));

import { mapSelectionStore } from "./sampleStore";
import { useManifestEditor } from "./useManifestEditor";

const document = (
	revision = "one",
	code = "original",
	writable = true,
): ManifestDocument => ({
	identifier: "main",
	key: "main",
	folderPath: "project",
	entrypoint: "index.scry",
	files: [{ path: "index.scry", content: code }],
	revision,
	writable,
	local: false,
});
const clients: QueryClient[] = [];
function setup() {
	const client = new QueryClient({
		defaultOptions: { queries: { retry: false, staleTime: 300_000 } },
	});
	clients.push(client);
	const wrapper = ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={client}>{children}</QueryClientProvider>
	);
	return { client, ...renderHook(() => useManifestEditor(true), { wrapper }) };
}
beforeEach(() => {
	vi.resetAllMocks();
	mapSelectionStore.set({ id: "main", key: "main" });
	mocks.read.mockResolvedValue({ manifestDocument: document() });
	mocks.run.mockResolvedValue({
		envelope: { files: [], manifests: [], forges: [], diagrams: [] },
		stdout: "",
	});
	mocks.save.mockResolvedValue({
		saveManifestDocument: document("two", "edited"),
	});
	mocks.blocks.mockResolvedValue({ blocks: [] });
});
afterEach(() => {
	cleanup();
	for (const client of clients.splice(0)) client.clear();
});

it("loads source, saves it, and refetches blocks even when their cache is fresh", async () => {
	const hook = setup();
	hook.client.setQueryData(["GetBlocks", { scryIdentifier: "main" }], {
		blocks: [{ name: "old" }],
	});
	await waitFor(() => expect(hook.result.current.code).toBe("original"));
	act(() => hook.result.current.setCode("edited"));
	mocks.read.mockResolvedValue({ manifestDocument: document("two", "edited") });
	await act(() => hook.result.current.run());
	expect(mocks.save).toHaveBeenCalledOnce();
	expect(mocks.blocks).toHaveBeenCalledOnce();
	expect(hook.result.current.status).toBe("Saved to cloud · Diagram updated");
	expect(hook.result.current.dirty).toBe(false);
});
it("preserves a dirty draft and blocks saving when the server revision changes", async () => {
	const hook = setup();
	await waitFor(() => expect(hook.result.current.code).toBe("original"));
	act(() => hook.result.current.setCode("my draft"));
	act(() =>
		hook.client.setQueryData(["ManifestDocument", "main", "local"], {
			manifestDocument: document("external", "external"),
		}),
	);
	await waitFor(() => expect(hook.result.current.conflict).toBe(true));
	expect(hook.result.current.code).toBe("my draft");
	await act(() => hook.result.current.run());
	expect(mocks.run).not.toHaveBeenCalled();
});
it("never runs or saves a read-only source", async () => {
	mocks.read.mockResolvedValue({
		manifestDocument: document("one", "original", false),
	});
	const hook = setup();
	await waitFor(() => expect(hook.result.current.doc).toBeDefined());
	await act(() => hook.result.current.run());
	expect(mocks.run).not.toHaveBeenCalled();
	expect(mocks.save).not.toHaveBeenCalled();
});
it("retains separate drafts when switching source documents", async () => {
	mocks.read.mockImplementation(async (id: string) => ({
		manifestDocument: {
			...document(),
			identifier: id,
			key: id,
			folderPath: id,
		},
	}));
	const hook = setup();
	await waitFor(() => expect(hook.result.current.code).toBe("original"));
	act(() => hook.result.current.setCode("main draft"));
	act(() => mapSelectionStore.set({ id: "other", key: "other" }));
	await waitFor(() =>
		expect(hook.result.current.doc?.identifier).toBe("other"),
	);
	expect(hook.result.current.code).toBe("original");
	act(() => hook.result.current.setCode("other draft"));
	act(() => mapSelectionStore.set({ id: "main", key: "main" }));
	await waitFor(() => expect(hook.result.current.code).toBe("main draft"));
	hook.unmount();
	const fresh = setup();
	await waitFor(() => expect(fresh.result.current.code).toBe("original"));
});
it("retains edits made while a save is in flight", async () => {
	let finish!: (value: { saveManifestDocument: ManifestDocument }) => void;
	mocks.save.mockImplementation(
		() =>
			new Promise((resolve) => {
				finish = resolve;
			}),
	);
	const hook = setup();
	await waitFor(() => expect(hook.result.current.code).toBe("original"));
	act(() => hook.result.current.setCode("edited"));
	let pending!: Promise<void>;
	act(() => {
		pending = hook.result.current.run();
	});
	await waitFor(() => expect(mocks.save).toHaveBeenCalledOnce());
	act(() => hook.result.current.setCode("newer draft"));
	mocks.read.mockResolvedValue({ manifestDocument: document("two", "edited") });
	await act(async () => {
		finish({ saveManifestDocument: document("two", "edited") });
		await pending;
	});
	expect(hook.result.current.code).toBe("newer draft");
	expect(hook.result.current.dirty).toBe(true);
	expect(hook.result.current.conflict).toBe(false);
});
