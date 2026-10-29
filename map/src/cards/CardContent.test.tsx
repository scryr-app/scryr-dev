// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, it } from "vite-plus/test";
import typescriptLogo from "../icons/typescript.svg?url";
import { InfoCard } from "./InfoCard";
import { OtherDiagramCard } from "./OtherDiagramCard";

afterEach(cleanup);

it("renders complete metadata and every resource without shortening the content", () => {
	const description =
		"This service has a detailed description with important deployment constraints. ".repeat(12);
	render(
		<InfoCard
			classification="Classification.public_api"
			description={description}
			version="2.3.4"
			languages={["typescript", "python"]}
			frameworks={["react", "nextjs", "express", "angular"]}
			deployments={["ec2", "terraform"]}
			owners="Platform engineering"
			metricTools={["prometheus", "jaeger", "loki", "prometheus"]}
			docs={["https://example.com/guide", "https://example.com/api"]}
			links={[
				{ siteName: "Source", httpUrl: "https://example.com/source" },
				{ siteName: "Status", httpUrl: "https://example.com/status" },
			]}
		/>,
	);
	const article = screen.getByRole("article", { name: "Service information" });
	expect(article.textContent).toContain(description.trim());
	expect(screen.getByText("v2.3.4")).toBeTruthy();
	expect(screen.getByText("Public API")).toBeTruthy();
	expect(screen.getByText("Platform engineering")).toBeTruthy();
	const stack = screen.getByRole("region", { name: "Stack" });
	expect(within(stack).getByText("Angular")).toBeTruthy();
	expect(within(stack).getByText("Python")).toBeTruthy();
	expect(within(stack).getByText("Terraform")).toBeTruthy();
	expect(within(stack).getAllByRole("listitem")).toHaveLength(8);
	expect(screen.queryByText("Auth")).toBeNull();
	expect(
		within(screen.getByRole("region", { name: "Metric tools" })).getAllByRole("listitem"),
	).toHaveLength(3);
	const links = within(screen.getByRole("navigation", { name: "Service resources" })).getAllByRole(
		"link",
	);
	expect(links).toHaveLength(4);
	for (const link of links) {
		expect(link.getAttribute("target")).toBe("_blank");
		expect(link.getAttribute("rel")).toBe("noopener noreferrer");
	}
});

it("uses browser SVG assets and renders unknown classifications without dropping their label", () => {
	const { container } = render(
		<InfoCard classification="custom_service" languages={["typescript"]} />,
	);
	expect(screen.getByText("Custom service")).toBeTruthy();
	expect(container.querySelector("svg path")).toBeTruthy();
	const logo = container.querySelector("img");
	expect(logo?.getAttribute("src")).toBe(typescriptLogo);
	expect(logo?.getAttribute("alt")).toBe("");
});

it("does not turn unsafe or malformed resource URLs into links", () => {
	render(
		<InfoCard
			docs={["javascript:alert(1)", "data:text/html,bad", "not a URL", "https://example.com/docs"]}
			links={[
				{ siteName: "Unsafe", httpUrl: "javascript:alert(1)" },
				{ siteName: "Local", httpUrl: "http://localhost:8000/health" },
			]}
		/>,
	);
	const links = within(screen.getByRole("navigation", { name: "Service resources" })).getAllByRole(
		"link",
	);
	expect(links.map((link) => link.getAttribute("href"))).toEqual([
		"https://example.com/docs",
		"http://localhost:8000/health",
	]);
	expect(screen.queryByRole("link", { name: "Unsafe" })).toBeNull();
});

it("renders empty cards and all unique diagram names as ordinary DOM", () => {
	const view = render(<InfoCard />);
	expect(screen.getByText("No service details provided.")).toBeTruthy();
	view.rerender(<OtherDiagramCard />);
	expect(screen.getByText("No diagrams found.")).toBeTruthy();
	view.rerender(<OtherDiagramCard diagrams={["Services", "Data", "Services", "  "]} />);
	expect(screen.getAllByRole("listitem")).toHaveLength(2);
	expect(screen.getByText("Services")).toBeTruthy();
	expect(screen.getByText("Data")).toBeTruthy();
});
