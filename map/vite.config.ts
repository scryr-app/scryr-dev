import { defineConfig } from "vite";
import { devtools } from "@tanstack/devtools-vite";
import viteReact from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

import { fileURLToPath, URL } from "node:url";

// https://vitejs.dev/config/
export default defineConfig({
	plugins: [
		devtools({
			injectSource: {
				enabled: false,
			},
		}),
		viteReact(),
		tailwindcss(),
	],
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
		// @react-three/uikit derives R3F element registration from runtime
		// component names. Keeping the production bundle unminified avoids
		// name mangling that can break UIKit rendering in `vite preview`.
		minify: "esbuild",
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
    	exclude: ['@react-three/uikit', '@pmndrs/uikit'],
	},
});
