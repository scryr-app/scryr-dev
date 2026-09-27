// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";

const transport = vi.hoisted(() => ({ request: vi.fn() }));
vi.mock("./client", () => ({ graphqlFetcher: () => transport.request }));

import { useBlocksData } from "./useBlocksData";
import { EditorScope } from "./useManifestEditor";

afterEach(cleanup);
it("does not reuse another organization's evidence for the same diagram identifier", async () => {
	const client = new QueryClient({
		defaultOptions: { queries: { retry: false, staleTime: 300_000 } },
	});
	let scope = "org-a";
	const wrapper = ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={client}>
			<EditorScope.Provider value={scope}>{children}</EditorScope.Provider>
		</QueryClientProvider>
	);
	const response = (name: string) => ({
		blocks: [
			{
				name,
				connections: [],
				tags: [],
				docs: [],
				frameworks: [],
				links: [],
				rawJsonString: "{}",
				evidence: [],
			},
		],
	});
	transport.request.mockResolvedValue(response("Private A"));
	const hook = renderHook(
		() => useBlocksData({ scryIdentifier: "shared-id" }, false),
		{ wrapper },
	);
	await waitFor(() =>
		expect(hook.result.current.blocks[0]?.name).toBe("Private A"),
	);
	let complete: (value: unknown) => void = () => {};
	transport.request.mockImplementation(
		() =>
			new Promise((resolve) => {
				complete = resolve;
			}),
	);
	scope = "org-b";
	hook.rerender();
	expect(hook.result.current.blocks).toEqual([]);
	await waitFor(() => expect(transport.request).toHaveBeenCalledTimes(2));
	await act(async () => complete(response("Private B")));
	await waitFor(() =>
		expect(hook.result.current.blocks[0]?.name).toBe("Private B"),
	);
	hook.unmount();
	client.clear();
});
