// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vite-plus/test";
import { MarkdownDescription } from "./MarkdownDescription";

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

it("renders common Markdown and GFM extensions as semantic content", () => {
	const { container } = render(
		<MarkdownDescription
			source={[
				"# Overview",
				"",
				"**Bold** and *emphasis* and ~~removed~~.",
				"",
				"- [x] Ready",
				"- [ ] Pending",
				"",
				"> Quoted",
				"",
				"```python",
				"print('hello')",
				"```",
				"",
				"| Name | State |",
				"| --- | --- |",
				"| API | Ready |",
				"",
				"https://example.com",
				"",
				"A note[^1].",
				"",
				"[^1]: Footnote text",
			].join("\n")}
		/>,
	);
	expect(screen.getByRole("heading", { name: "Overview" })).toBeTruthy();
	expect(container.querySelector("strong")?.textContent).toBe("Bold");
	expect(container.querySelector("em")?.textContent).toBe("emphasis");
	expect(container.querySelector("del")?.textContent).toBe("removed");
	expect(screen.getAllByRole("checkbox").map((box) => (box as HTMLInputElement).checked)).toEqual([
		true,
		false,
	]);
	expect(container.querySelector("blockquote")?.textContent).toContain("Quoted");
	expect(container.querySelector("pre code")?.textContent).toContain("print('hello')");
	expect(screen.getByRole("table")).toBeTruthy();
	expect(screen.getByRole("link", { name: "https://example.com" }).getAttribute("href")).toBe(
		"https://example.com",
	);
	expect(screen.getByText("Footnote text", { exact: false })).toBeTruthy();
	expect(screen.queryByRole("button")).toBeNull();
});

it("omits raw HTML and blocks script URLs while preserving safe links", () => {
	const { container } = render(
		<MarkdownDescription
			source={
				"<script>alert(1)</script>\n\n[bad](javascript:alert%281%29) [good](https://example.com)"
			}
		/>,
	);
	expect(container.querySelector("script")).toBeNull();
	expect(screen.getByText("bad").getAttribute("href")).toBe("");
	const link = screen.getByRole("link", { name: "good" });
	expect(link.getAttribute("target")).toBe("_blank");
	expect(link.getAttribute("rel")).toBe("noopener noreferrer");
});

it("allows a long preview to expand and collapse without cutting Markdown source", () => {
	vi.spyOn(HTMLElement.prototype, "scrollHeight", "get").mockReturnValue(400);
	const getStyle = window.getComputedStyle;
	vi.spyOn(window, "getComputedStyle").mockImplementation((element) => {
		const style = getStyle(element);
		style.lineHeight = "21px";
		return style;
	});
	render(<MarkdownDescription source={"**Complete content**\n\n".repeat(30)} />);
	const button = screen.getByRole("button", { name: "Show more" });
	expect(button.getAttribute("aria-expanded")).toBe("false");
	expect(document.getElementById(button.getAttribute("aria-controls")!)?.dataset.expanded).toBe(
		"false",
	);
	fireEvent.click(button);
	expect(screen.getByRole("button", { name: "Show less" }).getAttribute("aria-expanded")).toBe(
		"true",
	);
	expect(screen.getAllByText("Complete content")).toHaveLength(30);
	fireEvent.click(button);
	expect(button.getAttribute("aria-expanded")).toBe("false");
});
