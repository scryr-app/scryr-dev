import { readFileSync } from "node:fs";
import { describe, expect, it } from "vite-plus/test";
import { ClassificationIcons, getClassificationMetadata } from "./ClassificationIcons";
import { LogosDictionary } from "./LogosDictionary";

describe("classification icons", () => {
	it("covers every serialized Python classification with an icon and label", () => {
		const source = readFileSync(
			new URL("../../../manifest/scryr/src/scryr/saas/classification.py", import.meta.url),
			"utf8",
		);
		const values = [...source.matchAll(/^ {4}\w+ = "([^"]+)"/gm)].map((match) => match[1]);
		expect(values.length).toBeGreaterThan(0);
		for (const value of values) {
			expect(ClassificationIcons[value], value).toBeDefined();
			expect(ClassificationIcons[value].icon, value).toBeDefined();
			expect(ClassificationIcons[value].label, value).not.toBe("");
		}
	});

	it("resolves qualified names and the legacy gateway identifier", () => {
		expect(getClassificationMetadata("Classification.graphql")).toBe(ClassificationIcons.graphql);
		expect(getClassificationMetadata("public-api")).toBe(ClassificationIcons.public_api);
		expect(getClassificationMetadata("network_gateway")).toEqual(
			getClassificationMetadata("network_router"),
		);
	});

	it("keeps classification SQL separate from technology branding", () => {
		expect(getClassificationMetadata("sql").label).toBe("SQL");
		expect(getClassificationMetadata("sql")).not.toHaveProperty("iconFile");
		expect(LogosDictionary.sql.iconFile).toBe("postgresql.svg");
	});

	it("preserves custom labels with a neutral fallback icon", () => {
		expect(getClassificationMetadata("custom_service")).toEqual({
			icon: ClassificationIcons.unknown.icon,
			label: "Custom service",
		});
		expect(getClassificationMetadata("constructor").icon).toBe(ClassificationIcons.unknown.icon);
	});
});
