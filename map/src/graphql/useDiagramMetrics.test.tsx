// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

const request = vi.hoisted(() => vi.fn(async () => ({ diagramMetrics: {} })));
vi.mock("./client", () => ({ graphqlFetcher: () => request }));

import { diagramMetricsOptions, useDiagramMetrics } from "./useDiagramMetrics";

afterEach(() => {
	cleanup();
	request.mockClear();
});
describe("diagram-load collection", () => {
	it("fetches on opening and switching, but not on ordinary rerenders or polling", async () => {
		const client = new QueryClient();
		const wrapper = ({ children }: { children: ReactNode }) => (
			<QueryClientProvider client={client}>{children}</QueryClientProvider>
		);
		const hook = renderHook(
			({ id }) => useDiagramMetrics({ scryIdentifier: id }, true),
			{ wrapper, initialProps: { id: "northwind" } },
		);
		await waitFor(() => expect(hook.result.current.isSuccess).toBe(true));
		expect(request).toHaveBeenCalledTimes(1);
		hook.rerender({ id: "northwind" });
		hook.rerender({ id: "northwind" });
		expect(request).toHaveBeenCalledTimes(1);
		expect(diagramMetricsOptions.refetchInterval).toBe(false);
		expect(diagramMetricsOptions.refetchOnWindowFocus).toBe(false);
		expect(diagramMetricsOptions.refetchOnReconnect).toBe(false);
		hook.rerender({ id: "another" });
		await waitFor(() => expect(request).toHaveBeenCalledTimes(2));
		hook.unmount();
		client.clear();
	});
	it("does not fetch for offline previews or unconfigured diagrams", () => {
		const client = new QueryClient();
		renderHook(
			() => useDiagramMetrics({ scryIdentifier: "northwind" }, false),
			{
				wrapper: ({ children }) => (
					<QueryClientProvider client={client}>{children}</QueryClientProvider>
				),
			},
		);
		expect(request).not.toHaveBeenCalled();
		client.clear();
	});
});
