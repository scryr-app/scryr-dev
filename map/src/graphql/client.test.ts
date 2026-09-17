import { describe, expect, it } from "vitest";
import {
	localCapabilityHeaders,
	resolveGraphqlEndpoint,
	resolveSameOriginGraphqlEndpoint,
	shouldRetrySameOriginGraphqlEndpoint,
} from "./client";

describe("GraphQL client endpoint resolution", () => {
	it("resolves embedded relative endpoints against the served UI origin", () => {
		expect(resolveGraphqlEndpoint("/graphql", "http://127.0.0.1:8000")).toBe(
			"http://127.0.0.1:8000/graphql",
		);
	});

	it("keeps absolute development endpoints unchanged", () => {
		expect(
			resolveGraphqlEndpoint(
				"http://localhost:8000/graphql",
				"http://127.0.0.1:3000",
			),
		).toBe("http://localhost:8000/graphql");
	});

	it("can retry local-auth endpoints against the current UI origin", () => {
		expect(
			shouldRetrySameOriginGraphqlEndpoint(
				"http://localhost:8000/graphql",
				"local",
				"http://127.0.0.1:18080",
			),
		).toBe(true);
	});

	it("does not retry cloud auth endpoints against the current UI origin", () => {
		expect(
			shouldRetrySameOriginGraphqlEndpoint(
				"https://scryr.app/graphql",
				"clerk",
				"http://127.0.0.1:8000",
			),
		).toBe(false);
	});

	it("does not retry when the endpoint already matches same-origin GraphQL", () => {
		const endpoint = resolveSameOriginGraphqlEndpoint("http://127.0.0.1:8000");

		expect(
			shouldRetrySameOriginGraphqlEndpoint(
				endpoint,
				"local",
				"http://127.0.0.1:8000",
			),
		).toBe(false);
	});
});

describe("local collector capability transport", () => {
	it("sends the capability only to the originating loopback server", () => {
		expect(
			localCapabilityHeaders(
				"/graphql",
				"opaque-secret",
				"local",
				"http://127.0.0.1:8000",
			),
		).toEqual({ "X-Scryr-Local-Capability": "opaque-secret" });
		expect(
			localCapabilityHeaders(
				"/graphql",
				"opaque-secret",
				"local",
				"http://[::1]:8000",
			),
		).toEqual({ "X-Scryr-Local-Capability": "opaque-secret" });
	});
	it("does not expose local capabilities across ports, external hosts, or hosted auth", () => {
		expect(
			localCapabilityHeaders(
				"http://127.0.0.1:8000/graphql",
				"opaque-secret",
				"local",
				"http://127.0.0.1:3000",
			),
		).toEqual({});
		expect(
			localCapabilityHeaders(
				"https://example.com/graphql",
				"opaque-secret",
				"local",
				"http://127.0.0.1:8000",
			),
		).toEqual({});
		expect(
			localCapabilityHeaders(
				"/graphql",
				"opaque-secret",
				"clerk",
				"http://127.0.0.1:8000",
			),
		).toEqual({});
		expect(
			localCapabilityHeaders(
				"/graphql",
				"opaque-secret",
				"local",
				"https://example.com",
			),
		).toEqual({});
	});
});
