import { graphqlFetcher } from "./client";

export type ManifestDocument = {
	identifier: string;
	key: string;
	folderPath: string;
	entrypoint: string;
	files: { path: string; content: string }[];
	revision: string;
	writable: boolean;
	local: boolean;
};
export type ManifestEnvelope = {
	files: ManifestDocument["files"];
	manifests: Record<string, unknown>[];
	forges: Record<string, unknown>[];
	diagrams: Record<string, unknown>[];
};
export const documentKey = (identifier: string) =>
	["ManifestDocument", identifier] as const;
export const readDocument = (identifier: string) =>
	graphqlFetcher<
		{ manifestDocument: ManifestDocument },
		{ identifier: string }
	>(
		"query EditorSource($identifier: String!) { manifestDocument(identifier: $identifier) }",
		{ identifier },
	)();
export const saveDocument = (
	document: ManifestDocument,
	envelope: ManifestEnvelope,
) =>
	graphqlFetcher<
		{ saveManifestDocument: ManifestDocument },
		{ identifier: string; revision: string; envelope: ManifestEnvelope }
	>(
		"mutation SaveEditorSource($identifier: String!, $revision: String!, $envelope: JSON!) { saveManifestDocument(identifier: $identifier, revision: $revision, envelope: $envelope) }",
		{ identifier: document.identifier, revision: document.revision, envelope },
	)();
