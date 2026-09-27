import { beforeEach, expect, it, vi } from "vitest";

const transport = vi.hoisted(() => ({
	canApply: false,
	request: vi.fn(async () => ({ saveManifestDocument: { revision: "saved" } })),
}));
vi.mock("./client", () => ({
	canApplyLocalDocument: () => transport.canApply,
	graphqlFetcher: () => transport.request,
}));

import {
	type ManifestDocument,
	type ManifestEnvelope,
	saveDocument,
} from "./manifestDocument";

const document: ManifestDocument = {
	identifier: "diagram",
	key: "main",
	folderPath: ".",
	entrypoint: "index.scry",
	files: [{ path: "index.scry", content: "draft" }],
	revision: "revision",
	writable: true,
	local: true,
};
const envelope: ManifestEnvelope = {
	files: document.files,
	manifests: [],
	diagrams: [],
	forges: [],
};
beforeEach(() => {
	transport.canApply = false;
	transport.request.mockClear();
});
it("rejects local applies without a process capability before sending any source", async () => {
	await expect(saveDocument(document, envelope)).rejects.toThrow(
		"scryr serve address",
	);
	expect(transport.request).not.toHaveBeenCalled();
	expect(document.files[0].content).toBe("draft");
});
it("permits authorized local applies and preserves hosted snapshot saves", async () => {
	transport.canApply = true;
	await saveDocument(document, envelope);
	transport.canApply = false;
	await saveDocument({ ...document, local: false }, envelope);
	expect(transport.request).toHaveBeenCalledTimes(2);
});
