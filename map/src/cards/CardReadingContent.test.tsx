// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vite-plus/test";
import type { Block } from "@/graphql/generated";
import { DiagramCardReadingContent, InfoCardReadingContent } from "./CardReadingContent";

afterEach(cleanup);

const block: Block = {
	connections: [],
	docs: ["https://example.com/first", "https://example.com/second"],
	frameworks: ["react", "django", "flask", "fastapi"],
	links: [],
	rawJsonString: "{}",
	tags: [],
};

it("shows complete descriptions, every framework and documentation link", () => {
	const description = `${"A long service description. ".repeat(30)}Final sentence.`;
	render(
		<InfoCardReadingContent block={{ ...block, description, minReplicas: 0, maxReplicas: 12 }} />,
	);
	expect(screen.getByText(description)).toBeDefined();
	expect(screen.getByRole("link", { name: "FastAPI" })).toBeDefined();
	for (const url of block.docs) {
		expect(screen.getByRole("link", { name: url }).getAttribute("href")).toBe(url);
	}
	expect(screen.getByText("0")).toBeDefined();
	expect(screen.getByText("12")).toBeDefined();
});

it("keeps non-web URLs as text while preserving safe external links", () => {
	render(
		<InfoCardReadingContent
			block={{
				...block,
				frameworks: [],
				docs: ["javascript:alert(1)", "data:text/html,hello", "/relative"],
				links: [
					{ siteName: "Unsafe", httpUrl: "javascript:alert(1)" },
					{ siteName: "Safe", httpUrl: "https://example.com" },
				],
			}}
		/>,
	);
	expect(screen.getAllByRole("link")).toHaveLength(1);
	expect(screen.getByText("Unsafe").tagName).toBe("SPAN");
	expect(screen.getByRole("link", { name: "Safe" }).getAttribute("rel")).toBe(
		"noopener noreferrer",
	);
});

it("shows all diagram names and handles an empty list", () => {
	const { rerender } = render(
		<DiagramCardReadingContent diagrams={["Payments", "Platform", "Analytics"]} />,
	);
	expect(screen.getAllByRole("listitem")).toHaveLength(3);
	rerender(<DiagramCardReadingContent diagrams={[]} />);
	expect(screen.getByText("No diagrams found")).toBeDefined();
});
