// @vitest-environment jsdom
import {
	cleanup,
	fireEvent,
	render,
	screen,
	waitFor,
} from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { AccountButton } from "./AccountButton";

const signOut = vi.hoisted(() => vi.fn());
vi.mock("@clerk/react", () => ({
	UserAvatar: () => <span>Avatar</span>,
	useClerk: () => ({ signOut, openUserProfile: vi.fn() }),
}));
afterEach(() => {
	cleanup();
	vi.resetAllMocks();
});
it("logs out through Clerk and returns to the sign-in screen", async () => {
	render(<AccountButton />);
	fireEvent.click(screen.getByRole("button", { name: "Account menu" }));
	fireEvent.click(screen.getByRole("menuitem", { name: "Log out" }));
	await waitFor(() =>
		expect(signOut).toHaveBeenCalledWith({ redirectUrl: "/" }),
	);
});
it("allows retry when logout fails", async () => {
	signOut.mockRejectedValueOnce(new Error("offline"));
	render(<AccountButton />);
	fireEvent.click(screen.getByRole("button", { name: "Account menu" }));
	fireEvent.click(screen.getByRole("menuitem", { name: "Log out" }));
	expect(await screen.findByRole("alert")).toBeTruthy();
	expect(
		screen.getByRole("menuitem", { name: "Log out" }).hasAttribute("disabled"),
	).toBe(false);
});
