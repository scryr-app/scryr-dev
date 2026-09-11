import assert from "node:assert/strict";
import test from "node:test";
import { checkHostedConfig } from "./check-hosted-config.mjs";

const hosted = {
	VERCEL: "1",
	VITE_SCRYR_AUTH_MODE: "clerk",
	VITE_CLERK_PUBLISHABLE_KEY: "pk_test_fixture",
	VITE_GRAPHQL_ENDPOINT: "https://graphql.example.com/graphql",
};

test("local builds retain local authentication", () => {
	assert.doesNotThrow(() => checkHostedConfig({}));
});

test("properly configured hosted builds pass", () => {
	assert.doesNotThrow(() => checkHostedConfig(hosted));
});

test("hosted builds reject missing or local authentication", () => {
	for (const mode of [undefined, "local", ""]) {
		assert.throws(() => checkHostedConfig({ ...hosted, VITE_SCRYR_AUTH_MODE: mode }), /AUTH_MODE=clerk/);
	}
});

test("hosted builds require a publishable key and HTTPS API", () => {
	assert.throws(() => checkHostedConfig({ ...hosted, VITE_CLERK_PUBLISHABLE_KEY: "" }), /PUBLISHABLE_KEY/);
	for (const endpoint of [undefined, "/graphql", "http://localhost:8000/graphql"]) {
		assert.throws(() => checkHostedConfig({ ...hosted, VITE_GRAPHQL_ENDPOINT: endpoint }), /HTTPS/);
	}
});
