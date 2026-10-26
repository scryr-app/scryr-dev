import { defineConfig, lazyPlugins } from "vite-plus";
import { devtools } from "@tanstack/devtools-vite";
import viteReact from "@vitejs/plugin-react";

import { fileURLToPath, URL } from "node:url";
// https://vitejs.dev/config/
export default defineConfig({
	fmt: {
		useTabs: true,
		ignorePatterns: [
			"src/graphql/generated.ts",
			"src/pyodide/sdkSources.generated.json",
			"src/icons/**/*.svg",
			"package-lock.json",
		],
	},
	lint: {
		ignorePatterns: [
			"src/graphql/generated.ts",
			"src/pyodide/sdkSources.generated.json",
			"src/icons/**/*.svg",
		],
		jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
		rules: { "vite-plus/prefer-vite-plus-imports": "error" },
		options: { typeAware: true, typeCheck: true },
	},
	worker: { format: "es" },
	plugins: lazyPlugins(() => [
		devtools({
			injectSource: {
				enabled: false,
			},
		}),
		viteReact(),
	]),
	resolve: {
		alias: [
			{
				find: "@",
				replacement: fileURLToPath(new URL("./src", import.meta.url)),
			},
		],
	},
	server: {
		host: process.env.HOST ?? "localhost",
		port: Number(process.env.PORT ?? 3000),
		fs: {
			allow: [fileURLToPath(new URL("..", import.meta.url))],
		},
	},
	build: {
		chunkSizeWarningLimit: 20000,
	},
	define: {
		"process.env": {},
	},
	optimizeDeps: {
		rolldownOptions: {
			transform: {
				define: {
					global: "globalThis",
				},
			},
		},
	},
});
