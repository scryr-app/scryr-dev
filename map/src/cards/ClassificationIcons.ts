import { type ComponentType, type HTMLAttributes, createElement } from "react";
import styles from "./CardContent.module.css";

type ClassificationIconProps = HTMLAttributes<HTMLSpanElement> & {
	width?: number;
	height?: number;
};

type ClassificationMetadata = {
	icon: ComponentType<ClassificationIconProps>;
	label: string;
	iconWidth?: number;
};

const assets = import.meta.glob<string>("../icons/classifications/*.svg", {
	eager: true,
	import: "default",
	query: "?raw",
});

const definitions: Record<string, { asset: string; label: string; iconWidth?: number }> = {
	public_api: { asset: "public_api", label: "Public API" },
	internal_api: { asset: "internal_api", label: "Internal API" },
	network_router: { asset: "network_gateway", label: "Router" },
	public_ui: { asset: "public_ui", label: "Public UI" },
	mobile_app: { asset: "mobile_app", label: "Mobile app" },
	admin_ui: { asset: "admin_ui", label: "Admin UI" },
	cli: { asset: "cli", label: "CLI" },
	sdk: { asset: "sdk", label: "SDK" },
	datastore: { asset: "datastore", label: "Datastore" },
	database: { asset: "database", label: "Database" },
	object_storage: { asset: "object_storage", label: "Objects" },
	document_store: { asset: "document_store", label: "Documents" },
	vector_store: { asset: "vector_store", label: "Vectors" },
	columnar_store: { asset: "columnar_store", label: "Columns" },
	cache_store: { asset: "cache_store", label: "Cache" },
	search_store: { asset: "search_store", label: "Search" },
	queue: { asset: "queue", label: "Queue" },
	source: { asset: "source", label: "Source" },
	sink: { asset: "sink", label: "Sink" },
	worker: { asset: "worker", label: "Worker" },
	scheduler: { asset: "scheduler", label: "Scheduler" },
	job_processor: { asset: "job_processor", label: "Jobs" },
	web_ui: { asset: "web_ui", label: "Web UI" },
	mobile_ui: { asset: "mobile_ui", label: "Mobile UI" },
	desktop_ui: { asset: "desktop_ui", label: "Desktop UI" },
	rest: { asset: "rest", label: "REST" },
	graphql: { asset: "graphql", label: "GraphQL" },
	grpc: { asset: "grpc", label: "gRPC", iconWidth: 30 },
	websocket: { asset: "websocket", label: "WebSocket" },
	event: { asset: "event", label: "Event" },
	soap: { asset: "soap", label: "SOAP" },
	jsonrpc: { asset: "jsonrpc", label: "JSON-RPC" },
	webhook: { asset: "webhook", label: "Webhook" },
	sql: { asset: "sql", label: "SQL" },
	nosql: { asset: "nosql", label: "NoSQL" },
	graphql_datalayer: { asset: "graphql_datalayer", label: "GQL data" },
	orm: { asset: "orm", label: "ORM" },
	dataframe: { asset: "dataframe", label: "Dataframe" },
	object_store: { asset: "object_store", label: "Objects" },
	stream: { asset: "stream", label: "Stream" },
	cache: { asset: "cache", label: "Cache" },
	search: { asset: "search", label: "Search" },
	file_system: { asset: "file_system", label: "Files" },
	api_gateway: { asset: "api_gateway", label: "Gateway" },
	load_balancer: { asset: "load_balancer", label: "Balancer" },
	reverse_proxy: { asset: "reverse_proxy", label: "Proxy" },
	cdn: { asset: "cdn", label: "CDN" },
	dns: { asset: "dns", label: "DNS" },
	firewall: { asset: "firewall", label: "Firewall" },
	identity_provider: { asset: "identity_provider", label: "Identity" },
	secrets_manager: { asset: "secrets_manager", label: "Secrets" },
	data_pipeline: { asset: "data_pipeline", label: "Pipeline" },
	workflow_engine: { asset: "workflow_engine", label: "Workflow" },
	monitoring: { asset: "monitoring", label: "Monitor" },
	log_collector: { asset: "log_collector", label: "Logs" },
	model_service: { asset: "model_service", label: "ML model" },
	unknown: { asset: "unknown", label: "Unknown" },
};

export const ClassificationIcons: Record<string, ClassificationMetadata> = Object.fromEntries(
	Object.entries(definitions).map(([name, { asset, ...metadata }]) => {
		const content = assets[`../icons/classifications/${asset}.svg`];
		if (!content) throw new Error(`Missing classification icon: ${asset}`);
		function ClassificationIcon({
			width = 24,
			height = 24,
			style,
			...props
		}: ClassificationIconProps) {
			// These SVGs are trusted, bundled assets; inline markup preserves currentColor.
			return createElement("span", {
				...props,
				className: styles.classificationIcon,
				style: { width, height, ...style },
				"aria-hidden": true,
				dangerouslySetInnerHTML: { __html: content },
			});
		}
		return [name, { ...metadata, icon: ClassificationIcon }];
	}),
);

// The Python member is network_gateway; its stored identifier is network_router.
ClassificationIcons.network_gateway = ClassificationIcons.network_router;

export function getClassificationMetadata(value: string): ClassificationMetadata {
	const name = value.trim().split(".").pop() ?? "";
	const key = name.toLowerCase().replace(/[\s-]+/g, "_");
	if (Object.hasOwn(ClassificationIcons, key)) {
		return ClassificationIcons[key];
	}
	const label = name.replace(/[_-]+/g, " ").replace(/\s+/g, " ");
	return {
		icon: ClassificationIcons.unknown.icon,
		label: label ? label[0].toUpperCase() + label.slice(1) : "Unknown",
	};
}
