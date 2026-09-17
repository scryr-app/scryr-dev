// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import type { GetScryrMapsQuery } from "@/graphql/generated";
import { mapSelectionStore } from "@/graphql/sampleStore";
import { DiagramButton } from "./DiagramButton";

const state = vi.hoisted(() => ({ local: false }));
vi.mock("@/auth/env", () => ({
	get isLocalAuthMode() {
		return state.local;
	},
}));
const maps: GetScryrMapsQuery["scryrMaps"] = [
	{
		id: "calcom",
		key: "calcom",
		scryIdentifier: "calcom",
		name: "Cal.com",
		folderPath: "tests/samples/calcom",
	},
	{
		id: "mern-customer",
		key: "mern/customer",
		scryIdentifier: "mern-customer",
		name: "MERN Customer Path",
		folderPath: "tests/samples/mern",
	},
	{
		id: "custom",
		key: "custom",
		scryIdentifier: "custom",
		name: "My service",
		folderPath: "services/api",
	},
].map((map) => ({
	...map,
	clerkOrgId: "org",
	orgSlug: null,
	gitCommitSha: null,
	fileName: "index.scry",
	updatedAt: "",
}));
vi.mock("@/graphql/generated", () => ({
	useGetScryrMapsQuery: () => ({ data: { scryrMaps: maps } }),
}));
beforeEach(() => {
	state.local = false;
	window.history.replaceState({}, "", "/");
	mapSelectionStore.set({ id: null, key: "" });
});
afterEach(cleanup);

it("opens the cloud customer path and groups samples together without moving user diagrams", () => {
	render(<DiagramButton />);
	expect(mapSelectionStore.get().id).toBe("mern-customer");
	fireEvent.click(screen.getByRole("button", { name: /MERN Customer Path/ }));
	expect(screen.getAllByText("Samples")).toHaveLength(1);
	expect(
		screen.getByText("services/api", { selector: 'span[class="truncate"]' }),
	).toBeTruthy();
	fireEvent.click(screen.getByRole("button", { name: /My service/ }));
	expect(mapSelectionStore.get().id).toBe("custom");
	expect(window.location.search).toBe("?diagram=custom");
});
it("honors a deep link before the default", () => {
	window.history.replaceState({}, "", "/?diagram=calcom");
	render(<DiagramButton />);
	expect(mapSelectionStore.get().id).toBe("calcom");
});
it("preserves an existing selection", () => {
	mapSelectionStore.set({ id: "custom", key: "custom" });
	render(<DiagramButton />);
	expect(mapSelectionStore.get().id).toBe("custom");
});
it("keeps local diagram defaults and source folders", () => {
	state.local = true;
	render(<DiagramButton />);
	expect(mapSelectionStore.get().id).toBe("calcom");
	fireEvent.click(screen.getByRole("button", { name: /Cal.com/ }));
	expect(screen.queryByText("Samples")).toBeNull();
});
