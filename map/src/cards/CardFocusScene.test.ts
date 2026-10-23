import { Group, Mesh, Scene } from "three";
import { expect, it } from "vite-plus/test";
import { createCardForeground } from "./CardFocusScene";

it("moves only the original card tree to the foreground and restores every render layer", () => {
	const scene = new Scene();
	const block = new Group();
	const card = new Group();
	const text = new Mesh();
	scene.add(block);
	block.add(card);
	card.add(text);
	text.layers.enable(2);
	const textMask = text.layers.mask;
	const foreground = createCardForeground();
	foreground.activate(card);
	expect(card.parent).toBe(block);
	expect(card.children).toEqual([text]);
	expect(block.layers.mask).toBe(1);
	expect(card.layers.isEnabled(31)).toBe(true);
	expect(text.layers.isEnabled(0)).toBe(false);
	expect(foreground.contains(text)).toBe(true);
	expect(foreground.contains(block)).toBe(false);
	// Font/texture resources may finish mounting after the transition starts.
	const icon = new Mesh();
	card.add(icon);
	foreground.activate(card);
	expect(icon.layers.isEnabled(31)).toBe(true);
	foreground.release(card);
	expect(card.layers.mask).toBe(1);
	expect(text.layers.mask).toBe(textMask);
	expect(icon.layers.mask).toBe(1);
	expect(foreground.cards.size).toBe(0);
	expect(foreground.contains(text)).toBe(false);
});

it("returns one card without changing the other foreground cards", () => {
	const foreground = createCardForeground();
	const first = new Group();
	const second = new Group();
	foreground.activate(first);
	foreground.activate(second);
	foreground.release(first);
	expect(first.layers.isEnabled(0)).toBe(true);
	expect(second.layers.isEnabled(31)).toBe(true);
	expect(foreground.cards.size).toBe(1);
});
