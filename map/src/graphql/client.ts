/**
 * GraphQL client using graphql-request for React Query integration
 */
import {
	ClientError,
	GraphQLClient,
	type RequestDocument,
} from "graphql-request";
import {
	isLocalAuthMode,
	isScryrLocalAuthMode,
	runtimeConfig,
	scryrAuthMode,
} from "@/auth/env";

// VITE_GRAPHQL_ENDPOINT is baked in at build time.
// Set via the root `mise.toml` or a local override in untracked `mise.local.toml`.
const DEFAULT_GRAPHQL_PATH = "/graphql";
const CONFIGURED_GRAPHQL_ENDPOINT =
	runtimeConfig?.graphqlEndpoint ??
	import.meta.env.VITE_GRAPHQL_ENDPOINT ??
	DEFAULT_GRAPHQL_PATH;
const GRAPHQL_ENDPOINT = resolveGraphqlEndpoint(CONFIGURED_GRAPHQL_ENDPOINT);
type ClerkTokenGetter = () => Promise<string | null>;

let clerkTokenGetter: ClerkTokenGetter | null = null;

export function setClerkTokenGetter(getter: ClerkTokenGetter | null): void {
	clerkTokenGetter = getter;
}

/**
 * Singleton GraphQL client instance
 * Used by generated React Query hooks for type-safe GraphQL requests
 */
export const graphqlClient = new GraphQLClient(GRAPHQL_ENDPOINT, {
	headers: {
		"Content-Type": "application/json",
	},
});

function browserOrigin(): string | undefined {
	return typeof window === "undefined" ? undefined : window.location.origin;
}

export function resolveGraphqlEndpoint(
	configuredEndpoint: string | undefined,
	origin = browserOrigin(),
): string {
	const endpoint = configuredEndpoint?.trim() || DEFAULT_GRAPHQL_PATH;

	if (endpoint.startsWith("/") && origin) {
		return new URL(endpoint, origin).toString();
	}

	return endpoint;
}

export function resolveSameOriginGraphqlEndpoint(
	origin = browserOrigin(),
): string {
	return resolveGraphqlEndpoint(DEFAULT_GRAPHQL_PATH, origin);
}

export function shouldRetrySameOriginGraphqlEndpoint(
	endpoint: string,
	authMode = scryrAuthMode,
	origin = browserOrigin(),
): boolean {
	if (!isScryrLocalAuthMode(authMode) || !origin) {
		return false;
	}

	const sameOriginEndpoint = resolveSameOriginGraphqlEndpoint(origin);
	if (endpoint === sameOriginEndpoint) {
		return false;
	}

	try {
		const parsedEndpoint = new URL(endpoint, origin);
		return ["localhost", "127.0.0.1", "::1"].includes(parsedEndpoint.hostname);
	} catch {
		return false;
	}
}

/**
 * Custom fetcher function for React Query
 * This will be used by the generated hooks from graphql-codegen
 * Returns a function that returns a Promise (as required by React Query's queryFn)
 */
export function graphqlFetcher<TData, TVariables>(
	query: RequestDocument,
	variables?: TVariables,
) {
	return async (): Promise<TData> => {
		const clerkToken = await clerkTokenGetter?.();
		const headers: Record<string, string> | null = clerkToken
			? {
					Authorization: `Bearer ${clerkToken}`,
				}
			: null;

		if (!headers && !isLocalAuthMode) {
			throw new Error("Missing Clerk session token. Please sign in.");
		}

		try {
			return await graphqlClient.request<TData>(
				query,
				variables as Record<string, unknown>,
				headers ?? undefined,
			);
		} catch (error) {
			if (
				!(error instanceof ClientError) &&
				shouldRetrySameOriginGraphqlEndpoint(GRAPHQL_ENDPOINT)
			) {
				const fallbackClient = new GraphQLClient(
					resolveSameOriginGraphqlEndpoint(),
					{
						headers: {
							"Content-Type": "application/json",
						},
					},
				);
				return await fallbackClient.request<TData>(
					query,
					variables as Record<string, unknown>,
					headers ?? undefined,
				);
			}

			const message =
				error instanceof Error
					? error.message
					: "Unknown GraphQL request error.";
			if (error instanceof ClientError) {
				throw new Error(`GraphQL API returned an error: ${message}`);
			}
			throw new Error(
				`Failed to reach GraphQL API at ${GRAPHQL_ENDPOINT}: ${message}`,
			);
		}
	};
}
