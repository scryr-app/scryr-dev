// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vite-plus/test";
import { HtmlGraphContent } from "./HtmlGraphContent";

afterEach(() => {
	cleanup();
	vi.useRealTimers();
});

it("supports live updates, pause, sample selection, and editable notes", async () => {
	vi.useFakeTimers();
	const onFocus = vi.fn();
	const { container } = render(<HtmlGraphContent mode="Native HTML texture" onFocus={onFocus} />);
	const graph = () => container.querySelector("polyline")?.getAttribute("points");
	const before = graph();
	await act(() => vi.advanceTimersByTime(700));
	expect(graph()).not.toBe(before);
	fireEvent.click(screen.getByRole("button", { name: "Pause updates" }));
	const paused = graph();
	await act(() => vi.advanceTimersByTime(1400));
	expect(graph()).toBe(paused);
	fireEvent.change(screen.getByLabelText("Traffic"), { target: { value: "80" } });
	expect(graph()).not.toBe(paused);
	fireEvent.change(screen.getByLabelText("Inspect sample"), { target: { value: "2" } });
	expect(container.querySelectorAll("circle")[2]?.getAttribute("r")).toBe("7");
	fireEvent.change(screen.getByLabelText("Card notes"), { target: { value: "Persistent note" } });
	fireEvent.click(screen.getByRole("button", { name: "Enlarge card" }));
	expect(onFocus).toHaveBeenCalledOnce();
	expect((screen.getByLabelText("Card notes") as HTMLInputElement).value).toBe("Persistent note");
});
