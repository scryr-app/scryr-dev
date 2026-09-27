// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { MouseEventHandler, ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { EvidenceCard } from "./EvidenceCard";
import { EvidenceDetailsProvider } from "./EvidenceDetailsContext";
import { evidence, results } from "./evidenceFixtures";

vi.mock("@react-three/uikit", () => ({
	Container: ({
		children,
		onClick,
	}: {
		children: ReactNode;
		onClick?: MouseEventHandler;
	}) =>
		onClick ? (
			<button type="button" onClick={onClick}>
				{children}
			</button>
		) : (
			<div>{children}</div>
		),
	Text: ({ children }: { children: ReactNode }) => <span>{children}</span>,
}));
afterEach(cleanup);
it("keeps the chosen collector across polling reorder, then selects the remaining collector on removal", () => {
	const first = evidence();
	const second = evidence(results.coverage, {
		collectorId: "coverage",
		integration: "lcov",
	});
	const view = (collectors: (typeof first)[]) => (
		<EvidenceDetailsProvider>
			<EvidenceCard section="tests" collectors={collectors} />
		</EvidenceDetailsProvider>
	);
	const { rerender } = render(view([first, second]));
	expect(screen.getByText(/48 passed/)).toBeDefined();
	fireEvent.click(screen.getByRole("button", { name: "Next" }));
	expect(screen.getByText("unit coverage")).toBeDefined();
	rerender(view([second, first]));
	expect(screen.getByText("unit coverage")).toBeDefined();
	rerender(view([first]));
	expect(screen.getByText(/48 passed/)).toBeDefined();
	expect(screen.queryByRole("button", { name: "Next" })).toBeNull();
});
it("keeps retained findings visible when the next attempt fails", () => {
	const collector = evidence(results.check, {
		integration: "ruff",
		state: "ERROR" as ReturnType<typeof evidence>["state"],
		stale: true,
	});
	render(
		<EvidenceDetailsProvider>
			<EvidenceCard section="checks" collectors={[collector]} />
		</EvidenceDetailsProvider>,
	);
	expect(screen.getByText("Collection failed · Stale")).toBeDefined();
	expect(screen.getByText("Ruff: Failed")).toBeDefined();
});
it.each(["metrics", "repository"] as const)(
	"keeps incomplete %s warnings visible when results fill the compact card",
	(section) => {
		const result =
			section === "metrics"
				? {
						...results.metrics,
						complete: false,
						samples: [1, 2, 3, 4].map((index) => ({
							...results.metrics.samples[0],
							title: `Metric ${index}`,
						})),
					}
				: {
						...results.workflows,
						complete: false,
						items: [1, 2, 3].map((index) => ({
							...results.workflows.items[0],
							name: `Workflow ${index}`,
						})),
					};
		render(
			<EvidenceDetailsProvider>
				<EvidenceCard section={section} collectors={[evidence(result)]} />
			</EvidenceDetailsProvider>,
		);
		expect(screen.getByText("Collected")).toBeDefined();
		expect(
			screen.getByText(
				section === "metrics" ? /Partial scrape/ : /Partial listing/,
			),
		).toBeDefined();
		expect(
			screen.getByText(section === "metrics" ? /Metric 1/ : /Workflow 1/),
		).toBeDefined();
	},
);
