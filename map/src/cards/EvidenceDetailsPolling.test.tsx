// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
	act,
	cleanup,
	fireEvent,
	render,
	screen,
	waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

const transport = vi.hoisted(() => ({ request: vi.fn() }));
vi.mock("@/graphql/client", () => ({
	graphqlFetcher: () => transport.request,
}));
vi.mock("@/graphql/sampleStore", () => ({
	useSelectedMap: () => ({ id: "test-diagram" }),
}));

import { EditorScope } from "@/graphql/useManifestEditor";
import { EvidenceDetails } from "./EvidenceDetails";
import {
	EvidenceDetailsProvider,
	useEvidenceDetails,
} from "./EvidenceDetailsContext";
import { collectorKey } from "./evidence";
import { evidence, results } from "./evidenceFixtures";

beforeEach(() => {
	HTMLDialogElement.prototype.showModal = function () {
		this.setAttribute("open", "");
	};
	HTMLDialogElement.prototype.close = function () {
		this.removeAttribute("open");
	};
	transport.request.mockReset();
});
afterEach(cleanup);

it("retains newer cached evidence on refresh failure and clears the warning after retry", async () => {
	const client = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	});
	const opening = evidence({ ...results.tests, passing: 10 });
	const updated = evidence({ ...results.tests, passing: 20 }, { stale: true });
	const recovered = evidence({ ...results.tests, passing: 30 });
	const response = (collector: typeof opening) => ({
		blocks: [
			{
				name: "API",
				connections: [],
				tags: [],
				docs: [],
				frameworks: [],
				links: [],
				rawJsonString: JSON.stringify({
					manifestId: opening.manifestId,
					tests: [{ kind: opening.integration, id: opening.collectorId }],
				}),
				evidence: [collector],
			},
		],
	});
	function OpenDetails() {
		const { open } = useEvidenceDetails();
		return (
			<button
				type="button"
				onClick={() =>
					open({
						section: "tests",
						collectors: [opening],
						selectedId: collectorKey(opening),
					})
				}
			>
				Open details
			</button>
		);
	}
	transport.request.mockResolvedValue(response(opening));
	const view = render(
		<QueryClientProvider client={client}>
			<EditorScope.Provider value="test-org">
				<EvidenceDetailsProvider>
					<OpenDetails />
					<EvidenceDetails />
				</EvidenceDetailsProvider>
			</EditorScope.Provider>
		</QueryClientProvider>,
	);
	fireEvent.click(screen.getByRole("button", { name: "Open details" }));
	await waitFor(() =>
		expect(screen.getByText(/Suite unit: 10 passed/)).toBeDefined(),
	);
	await waitFor(() => expect(transport.request).toHaveBeenCalledTimes(1));
	transport.request.mockResolvedValue(response(updated));
	await act(async () => {
		await client.refetchQueries({ queryKey: ["GetBlocks"] });
	});
	await waitFor(() =>
		expect(screen.getByText(/Suite unit: 20 passed/)).toBeDefined(),
	);
	transport.request.mockRejectedValue(new Error("Connection lost"));
	await act(async () => {
		await client.refetchQueries({ queryKey: ["GetBlocks"] });
	});
	await waitFor(() =>
		expect(screen.getByRole("alert").textContent).toContain(
			"current freshness is unknown",
		),
	);
	expect(screen.getByText(/Suite unit: 20 passed/)).toBeDefined();
	expect(screen.queryByText(/Suite unit: 10 passed/)).toBeNull();
	expect(screen.getByText("Last known: Collected · Stale")).toBeDefined();
	transport.request.mockResolvedValue(response(recovered));
	fireEvent.click(screen.getByRole("button", { name: "Retry" }));
	await waitFor(() =>
		expect(screen.getByText(/Suite unit: 30 passed/)).toBeDefined(),
	);
	expect(screen.queryByRole("alert")).toBeNull();
	view.unmount();
	client.clear();
});
