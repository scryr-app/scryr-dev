import { ClerkProvider } from "@clerk/react";
import type { ThreeElements } from "@react-three/fiber";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import {
	createRootRoute,
	createRoute,
	createRouter,
	Outlet,
	RouterProvider,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import type { JSX, PropsWithChildren } from "react";
import { StrictMode } from "react";
import ReactDOM from "react-dom/client";

import "./styles.css";

import App from "./App.tsx";
import { isLocalAuthMode } from "./auth/env.ts";
import { isDebug } from "./utils/debug.ts";

const match = <Case extends string, Result>(
	caseKey: Case,
	cases: Record<Case, () => Result>,
): Result => cases[caseKey]();

// Create a QueryClient instance
const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			refetchOnWindowFocus: false,
			retry: 1,
			staleTime: 5 * 60 * 1000, // 5 minutes
		},
	},
});

// Handle type error with React 19 and threeJS react-three-fiber
declare global {
	namespace React {
		namespace JSX {
			interface IntrinsicElements extends ThreeElements {}
		}
	}
}

const RouterDevtools = () =>
	match(isDebug ? "enabled" : "disabled", {
		enabled: () => (
			<>
				<ReactQueryDevtools buttonPosition="bottom-left" />
				<TanStackRouterDevtools position="bottom-left" />
			</>
		),
		disabled: () => null,
	});

const createAppRouter = () => {
	const rootRoute = createRootRoute({
		component: () => (
			<>
				<Outlet />
				<RouterDevtools />
			</>
		),
	});

	return createRouter({
		routeTree: rootRoute.addChildren([
			createRoute({
				getParentRoute: () => rootRoute,
				path: "/",
				component: App,
			}),
		]),
		context: {},
		defaultPreload: "intent",
		scrollRestoration: true,
		defaultStructuralSharing: true,
		defaultPreloadStaleTime: 0,
	});
};

const router = createAppRouter();

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}

type EnvClerkProviderProps = PropsWithChildren<{
	afterSignOutUrl?: string;
}>;

// Clerk's current React quickstart relies on Vite env discovery, but the
// published 6.7.2 types still require publishableKey at compile time.
const EnvClerkProvider = ClerkProvider as unknown as (
	props: EnvClerkProviderProps,
) => JSX.Element;

const LocalClerkProvider = ({ children }: EnvClerkProviderProps) => (
	<>{children}</>
);

const AppClerkProvider = match(isLocalAuthMode ? "local" : "clerk", {
	local: () => LocalClerkProvider,
	clerk: () => EnvClerkProvider,
});

const rootElement = document.getElementById("app");
if (rootElement && !rootElement.innerHTML) {
	const root = ReactDOM.createRoot(rootElement);
	root.render(
		<StrictMode>
			<AppClerkProvider afterSignOutUrl="/">
				<QueryClientProvider client={queryClient}>
					<RouterProvider router={router} />
				</QueryClientProvider>
			</AppClerkProvider>
		</StrictMode>,
	);
}
