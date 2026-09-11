export function checkHostedConfig(env = process.env) {
	if (env.VERCEL !== "1") return;
	if (env.VITE_SCRYR_AUTH_MODE !== "clerk") {
		throw new Error("Vercel builds require VITE_SCRYR_AUTH_MODE=clerk.");
	}
	if (!env.VITE_CLERK_PUBLISHABLE_KEY?.startsWith("pk_")) {
		throw new Error("Vercel builds require VITE_CLERK_PUBLISHABLE_KEY.");
	}
	let endpoint;
	try {
		endpoint = new URL(env.VITE_GRAPHQL_ENDPOINT);
	} catch {
		throw new Error("Vercel builds require an absolute HTTPS VITE_GRAPHQL_ENDPOINT.");
	}
	if (endpoint.protocol !== "https:") {
		throw new Error("Vercel builds require an absolute HTTPS VITE_GRAPHQL_ENDPOINT.");
	}
}

checkHostedConfig();
