import { test, expect, type Locator, type Page, type TestInfo } from "@playwright/test";
import { createServer, type Server } from "node:http";
import { readFile } from "node:fs/promises";
import { resolve, extname } from "node:path";
import { fileURLToPath } from "node:url";

const dist = fileURLToPath(new URL("../dist/", import.meta.url));
let server: Server;
let base: string;
const description =
	"Deployment requires coordinated releases and careful monitoring across regions. ".repeat(70);
const blocks = ["Orders API", "Payments API"].map((name, index) => ({
	name,
	description: index === 0 ? description : "Processes payments and publishes settlement events.",
	version: "2.3.4",
	consumerType: "public_api",
	languages: ["typescript"],
	frameworks: ["react", "express"],
	deployments: ["ec2", "terraform"],
	owners: "Platform engineering",
	metricTools: ["prometheus", "jaeger", "loki"],
	connections: index === 0 ? ["Payments API"] : [],
	docs: ["https://example.com/docs"],
	tags: ["Commerce", "Platform"],
	links: [{ siteName: "Status", httpUrl: "https://example.com/status" }],
	rawJsonString: "{}",
}));

function surface(page: Page, block: string, index = 0) {
	return page.locator(`[data-card-id=${JSON.stringify(JSON.stringify([block, index]))}]`);
}

async function settled(card: Locator) {
	await expect
		.poll(async () => {
			const box = await card.boundingBox();
			return box !== null && box.width > 20 && box.x > 0 && box.y > 0;
		})
		.toBe(true);
	let previous = await card.boundingBox();
	let stableSamples = 0;
	await expect
		.poll(
			async () => {
				const current = await card.boundingBox();
				if (!current || !previous) {
					previous = current;
					return Infinity;
				}
				const distance =
					Math.abs(current.x - previous.x) +
					Math.abs(current.y - previous.y) +
					Math.abs(current.width - previous.width);
				previous = current;
				stableSamples = distance < 0.5 ? stableSamples + 1 : 0;
				return stableSamples;
			},
			{ intervals: [250], timeout: 15_000 },
		)
		.toBeGreaterThanOrEqual(4);
}

async function focus(page: Page, card: Locator) {
	await settled(card);
	await page.getByRole("button", { name: "Enlarge Card", exact: true }).click();
	await settled(card);
	const box = await card.boundingBox();
	if (!box) throw new Error("Card has no projected bounds");
	await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
	await expect(card).toHaveAttribute("data-card-focused", "true");
	await settled(card);
}

async function capture(page: Page, testInfo: TestInfo, name: string) {
	const path = testInfo.outputPath(`${name}.png`);
	await page.screenshot({ path });
	await testInfo.attach(name, { path, contentType: "image/png" });
}

// This fixture exercises the production build without touching SQLite, a running
// development stack, or any external service. The renderer needs no experimental browser APIs.
test.beforeAll(async () => {
	server = createServer(async (request, response) => {
		try {
			const pathname = new URL(request.url ?? "/", "http://localhost").pathname;
			const file = resolve(dist, pathname === "/" ? "index.html" : `.${pathname}`);
			if (!file.startsWith(dist)) {
				response.writeHead(403).end();
				return;
			}
			const contentTypes: Record<string, string> = {
				".html": "text/html",
				".js": "text/javascript",
				".css": "text/css",
				".svg": "image/svg+xml",
				".woff2": "font/woff2",
				".png": "image/png",
			};
			response.setHeader("Content-Type", contentTypes[extname(file)] ?? "application/octet-stream");
			response.end(await readFile(file));
		} catch {
			response.writeHead(404).end();
		}
	});
	await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
	const address = server.address();
	if (!address || typeof address === "string") throw new Error("No preview port");
	base = `http://127.0.0.1:${address.port}`;
});

test.afterAll(async () => {
	await new Promise<void>((resolve, reject) =>
		server.close((error) => (error ? reject(error) : resolve())),
	);
});

test("all cards use DOM, reflow in focus, compare, scroll, and return without remounting", async ({
	page,
}, testInfo) => {
	const errors: string[] = [];
	page.on("pageerror", (error) => errors.push(error.message));
	await page.route("**/graphql", async (route) => {
		const query = route.request().postDataJSON().query as string;
		const data = query.includes("GetBlocks")
			? { blocks }
			: query.includes("GetScryrMaps")
				? { scryrMaps: [] }
				: query.includes("HealthCheck")
					? { health: { status: "ok", databaseOk: true } }
					: { manifestDocument: null };
		await route.fulfill({ json: { data } });
	});
	await page.goto(base);
	await page.getByRole("button", { name: "Hide terminal file" }).click();
	const orders = surface(page, "Orders API");
	const payments = surface(page, "Payments API");
	await expect(page.locator("[data-card-id]")).toHaveCount(4, { timeout: 60_000 });
	await settled(orders);
	await expect(page.locator("canvas[layoutsubtree], canvas[content='drawable']")).toHaveCount(0);
	await expect(page.locator("canvas [data-card-id]")).toHaveCount(0);
	for (const card of await page.locator("[data-card-id]").all()) {
		await expect(card).toHaveAttribute("inert", "");
		await expect(card).toHaveCSS("pointer-events", "none");
	}
	await expect(orders).toHaveAttribute("data-card-layout-width", "420");
	await orders.evaluate((element) => {
		(window as unknown as { savedCard: Element }).savedCard = element;
	});
	await capture(page, testInfo, "docked-cards");

	// Blending disables pointer events on the canvas itself; navigation must
	// still arrive through its shared scene parent.
	const transform = () =>
		orders.evaluate((element) => {
			const transforms: string[] = [];
			for (let ancestor = element.parentElement; ancestor; ancestor = ancestor.parentElement) {
				if (ancestor.style.transform) transforms.push(ancestor.style.transform);
			}
			return transforms.join(" ");
		});
	const beforeOrbit = await transform();
	await page.mouse.move(1450, 750);
	await page.mouse.down();
	await page.mouse.move(1390, 750, { steps: 8 });
	await page.mouse.up();
	await expect.poll(transform).not.toBe(beforeOrbit);
	await settled(orders);

	await focus(page, orders);
	await expect(orders).not.toHaveAttribute("inert", "");
	await expect(orders.getByRole("article", { name: "Service information" })).toBeVisible();
	await expect
		.poll(async () => Number(await orders.getAttribute("data-card-layout-width")))
		.toBeGreaterThan(640);
	await expect(
		orders.getByText("Deployment requires coordinated releases", { exact: false }),
	).toContainText(description.trim());
	const showMore = orders.getByRole("button", { name: "Show more", exact: true });
	await expect(showMore).toHaveAttribute("aria-expanded", "false");
	const previewId = await showMore.getAttribute("aria-controls");
	const preview = orders.locator(`[id=${JSON.stringify(previewId)}]`);
	expect(await preview.evaluate((element) => element.scrollHeight > element.clientHeight)).toBe(
		true,
	);
	await showMore.click();
	await expect(orders.getByRole("button", { name: "Show less" })).toHaveAttribute(
		"aria-expanded",
		"true",
	);
	expect(
		await preview.evaluate((element) => element.scrollHeight <= element.clientHeight + 1),
	).toBe(true);
	const resources = orders.getByRole("navigation", { name: "Service resources" });
	await expect(resources.getByRole("link", { name: "Docs" })).toHaveAttribute(
		"href",
		"https://example.com/docs",
	);
	await expect(resources.getByRole("link", { name: "Docs" })).toHaveAttribute(
		"rel",
		"noopener noreferrer",
	);
	const wideColumns = await orders
		.locator("article > div")
		.first()
		.evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length);
	expect(wideColumns).toBe(2);
	await capture(page, testInfo, "focused-reflow");
	const box = await orders.boundingBox();
	if (!box) throw new Error("Focused card has no bounds");
	await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
	await page.mouse.wheel(0, 400);
	await expect.poll(() => orders.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
	await capture(page, testInfo, "focused-scrolling");

	await page.getByRole("button", { name: "Add card", exact: true }).click();
	await page.getByRole("button", { name: "Payments API — Info", exact: true }).click();
	await page.getByRole("button", { name: "Close card picker" }).click();
	await expect(payments).toHaveAttribute("data-card-focused", "true");
	await expect(page.getByRole("list", { name: "Focused cards" }).getByRole("listitem")).toHaveCount(
		2,
	);
	const comparisonWidth = Number(await orders.getAttribute("data-card-layout-width"));
	await page.getByRole("button", { name: "Expand Orders API — Info", exact: true }).click();
	await expect(payments).toHaveAttribute("inert", "");
	await expect
		.poll(async () => Number(await orders.getAttribute("data-card-layout-width")))
		.toBeGreaterThan(comparisonWidth);
	await page.getByRole("button", { name: "Back to group", exact: true }).click();
	await expect(payments).not.toHaveAttribute("inert", "");
	await expect(page.getByRole("list", { name: "Focused cards" }).getByRole("listitem")).toHaveCount(
		2,
	);

	await page.setViewportSize({ width: 390, height: 844 });
	await expect
		.poll(async () => Number(await orders.getAttribute("data-card-layout-width")))
		.toBeLessThan(640);
	await expect
		.poll(() =>
			orders
				.locator("article > div")
				.first()
				.evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length),
		)
		.toBe(1);
	await settled(orders);
	await settled(payments);
	await capture(page, testInfo, "mobile-comparison");
	await page.setViewportSize({ width: 1600, height: 1000 });
	await orders.evaluate((element) => {
		element.scrollTop = 40;
	});
	await page.getByRole("button", { name: "Back to diagram", exact: true }).click();
	await expect(orders).toHaveAttribute("data-card-focused", "false");
	await expect(orders).toHaveAttribute("inert", "");
	await expect(orders).toHaveAttribute("data-card-layout-width", "420");
	expect(
		await orders.evaluate(
			(element) => (window as unknown as { savedCard: Element }).savedCard === element,
		),
	).toBe(true);
	await settled(orders);
	await expect.poll(() => orders.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
	// Inert cards, including the covered cards in slots, cannot steal scene input.
	expect(
		await orders.evaluate((element) => {
			const bounds = element.getBoundingClientRect();
			return (
				document
					.elementFromPoint(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2)
					?.closest("[data-card-id]") == null
			);
		}),
	).toBe(true);
	await capture(page, testInfo, "returned-cards");
	await page.getByRole("button", { name: "Diagrams", exact: true }).click();
	const diagrams = surface(page, "Orders API", 1);
	await focus(page, diagrams);
	await expect(diagrams.getByRole("article", { name: "Other diagrams" })).toBeVisible();
	await expect(diagrams.getByText("Commerce", { exact: true })).toBeVisible();
	await capture(page, testInfo, "diagram-card");
	expect(errors).toEqual([]);
});
