// @vitest-environment jsdom
import { cleanup, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
import App from "./App";

const session = vi.hoisted(() => ({ local: false, signedIn: false }));
vi.mock("@clerk/react", () => ({
	Show: ({ when, children }: { when: string; children: ReactNode }) =>
		(when === "signed-in") === session.signedIn ? children : null,
	RedirectToSignIn: () => <div>Clerk redirect</div>,
}));
vi.mock("@/auth", async () => ({
	get isLocalAuthMode() {
		return session.local;
	},
	AuthenticatedSession: ({ children }: { children: ReactNode }) => children,
	OrganizationAccountControls: () => null,
	SignedOutScreen: (await import("./auth/SignedOutScreen")).SignedOutScreen,
}));
vi.mock("@/scene", () => ({ MapSceneShell: () => <div>Map</div> }));
vi.mock("@/components/OpenSourcePrompt", () => ({
	OpenSourcePrompt: () => null,
}));
afterEach(cleanup);
it("redirects signed-out cloud visitors to Clerk", () => {
	session.local = false;
	session.signedIn = false;
	render(<App />);
	expect(screen.getByText("Clerk redirect")).toBeDefined();
	expect(screen.queryByText("Map")).toBeNull();
});
it("opens local maps without Clerk or a session", () => {
	session.local = true;
	session.signedIn = false;
	render(<App />);
	expect(screen.getByText("Map")).toBeDefined();
	expect(screen.queryByText("Clerk redirect")).toBeNull();
});
it("opens authenticated cloud maps without redirecting", () => {
	session.local = false;
	session.signedIn = true;
	render(<App />);
	expect(screen.getByText("Map")).toBeDefined();
	expect(screen.queryByText("Clerk redirect")).toBeNull();
});
