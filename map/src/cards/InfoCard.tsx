import { Container, Svg, Text } from "@react-three/uikit";
import { Activity, Code, Info, Users } from "@react-three/uikit-lucide";
import { type ReactNode, useState } from "react";
import type { BlockLink } from "@/block/Block";
import { currentTheme } from "@/theme/theme";
import { type LogoMetadata, LogosDictionary } from "./LogosDictionary";

// Dimensions match Block defaults: width(3) * 0.8, height(2) * 0.8
const CARD_SIZE_X = 2.8;
const CARD_SIZE_Y = 1.8;
// 1 pixel = 0.01 world units -> 240x160 virtual pixel space
const PIXEL_SIZE = 0.01;
const INSET_BG = "rgba(0,0,0,0.22)";
const LABEL_COLOR = "rgba(255,255,255,0.40)";
const TOOLTIP_BG = "rgba(13,17,23,0.5)";
const TOOLTIP_BORDER = "rgba(255,255,255,0.22)";
const LINK_COLOR = "#93c5fd";
const LINK_CHIP_BG = "rgba(147,197,253,0.16)";
const LINK_CHIP_BORDER = "rgba(147,197,253,0.34)";
const META_CHIP_BG = "rgba(255,255,255,0.10)";
const META_CHIP_BORDER = "rgba(255,255,255,0.16)";
const OWNER_CHIP_BG = "rgba(255,255,255,0.08)";
const OWNER_CHIP_BORDER = "rgba(255,255,255,0.12)";

const CARD_PADDING = 7;
const CARD_GAP = 4;
const HEADER_HEIGHT = 18;
const INFO_PANEL_MIN_HEIGHT = 28;
const INFO_PANEL_MAX_HEIGHT = 86;
const DETAIL_ROW_HEIGHT = 24;
const ROW_LABEL_WIDTH = 40;
const SECTION_PADDING = 5;
const CONTENT_WIDTH = CARD_SIZE_X / PIXEL_SIZE - CARD_PADDING * 2;
const DESCRIPTION_TEXT_WIDTH = CONTENT_WIDTH - SECTION_PADDING * 2;
const ROW_LABEL_ICON_SIZE = 7;
const ENUM_ICON_SIZE = 15;
const ENUM_CHIP_SIZE = 20;
const META_ICON_SIZE = 11;
const META_PILL_HEIGHT = 20;
const TYPE_PILL_WIDTH = 76;
const AUTH_PILL_WIDTH = 66;
const OWNER_CHIP_WIDTH = 86;
const ENUM_TOOLTIP_WIDTH = 116;
const DESCRIPTION_TOOLTIP_WIDTH = 216;
const DESCRIPTION_TOOLTIP_LEFT =
	(DESCRIPTION_TEXT_WIDTH - DESCRIPTION_TOOLTIP_WIDTH) / 2;
const DESCRIPTION_LINE_LENGTH = 56;
const DESCRIPTION_LINE_HEIGHT = 12;
const DESCRIPTION_MAX_LINES = 3;
const TOOLTIP_RADIUS = 10;
const TOOLTIP_Z_OFFSET = 1000;
const TOOLTIP_Z_TRANSLATE = 64;
const ENUM_ICON_OPACITY = 0.62;
const MUTED_ENUM_ICON_OPACITY = 0.5;

function escapeRegExp(value: string): string {
	return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function fallbackGradientColor(svg: string, id: string): string {
	const gradientPattern = new RegExp(
		`<(?:linearGradient|radialGradient)\\b[^>]*\\bid=["']${escapeRegExp(
			id,
		)}["'][\\s\\S]*?</(?:linearGradient|radialGradient)>`,
	);
	const gradient = gradientPattern.exec(svg)?.[0] ?? "";
	return (
		/(?:stop-color)=["']([^"']+)["']/.exec(gradient)?.[1] ??
		/stop-color:\s*([^;"']+)/.exec(gradient)?.[1] ??
		currentTheme.cardTextColor
	);
}

function sanitizeSvgForThree(svg: string): string {
	return svg
		.replace(/<script\b[\s\S]*?<\/script>/g, "")
		.replace(/\b(fill|stroke)=["']url\(#([^)]+)\)["']/g, (_match, attr, id) => {
			return `${attr}="${fallbackGradientColor(svg, id)}"`;
		})
		.replace(/\b(fill|stroke):\s*url\(#([^)]+)\)/g, (_match, attr, id) => {
			return `${attr}: ${fallbackGradientColor(svg, id)}`;
		})
		.replace(/\s(?:clip-path|mask|filter)=["']url\([^)]+\)["']/g, "");
}

const ICON_CONTENT_BY_FILE = Object.fromEntries(
	Object.entries(
		import.meta.glob("../icons/*.svg", {
			eager: true,
			import: "default",
			query: "?raw",
		}),
	).map(([path, content]) => [
		path.split("/").pop() ?? path,
		sanitizeSvgForThree(content as string),
	]),
);

function normalizeEnumValue(value: string): string {
	const rawValue = value.includes(".")
		? value.split(".").pop() || value
		: value;
	return rawValue
		.trim()
		.toLowerCase()
		.replace(/[\s-]+/g, "_");
}

function getEnumMetadata(value: string): LogoMetadata | undefined {
	return LogosDictionary[normalizeEnumValue(value)];
}

function getEnumIconContent(value: string): string | null {
	const iconFile = getEnumMetadata(value)?.iconFile;
	return iconFile ? (ICON_CONTENT_BY_FILE[iconFile] ?? null) : null;
}

function getEnumLabel(value: string): string {
	const normalized = normalizeEnumValue(value);
	const label = LogosDictionary[normalized]?.label;
	if (label) return label;

	return normalized
		.split("_")
		.filter(Boolean)
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(" ");
}

function getEnumDocUrl(value: string): string | null {
	return getEnumMetadata(value)?.docUrl ?? null;
}

function getShortEnumLabel(value: string): string {
	const normalized = normalizeEnumValue(value);
	const parts = normalized.split("_").filter(Boolean);
	if (parts.length === 0) return "?";

	const label =
		parts.length === 1
			? parts[0].replace(/[aeiou]/g, "").slice(0, 3) || parts[0].slice(0, 2)
			: parts.map((part) => part[0]).join("");

	return label.slice(0, 3).toUpperCase();
}

function getMetaLabel(value: string): string {
	const normalized = normalizeEnumValue(value);
	const compactLabels: Record<string, string> = {
		admin_ui: "Admin UI",
		api_key: "API Key",
		auth_api_key: "Key",
		auth_basic: "Basic",
		auth_jwt: "JWT",
		auth_mutual_tls: "mTLS",
		auth_none: "None",
		auth_oauth2: "OAuth 2",
		auth_service_account: "Svc",
		cache_store: "Cache",
		columnar_store: "Columnar",
		database: "Database",
		datastore: "Datastore",
		document_store: "Docs",
		internal_api: "Int API",
		job_processor: "Jobs",
		mobile_app: "Mobile",
		network_gateway: "Network",
		network_router: "Network",
		object_storage: "Obj",
		public_api: "Pub API",
		public_ui: "Web UI",
		search_store: "Search",
		vector_store: "Vector",
	};

	return compactLabels[normalized] ?? getEnumLabel(value);
}

function EnumTooltip({
	label,
	docUrl,
	anchorWidth = ENUM_CHIP_SIZE,
}: {
	label: string;
	docUrl: string | null;
	anchorWidth?: number;
}) {
	return (
		<Container
			positionType="absolute"
			positionTop={-40}
			positionLeft={-(ENUM_TOOLTIP_WIDTH - anchorWidth) / 2}
			width={ENUM_TOOLTIP_WIDTH}
			flexDirection="column"
			alignItems="center"
			justifyContent="center"
			backgroundColor={TOOLTIP_BG}
			borderColor={TOOLTIP_BORDER}
			borderWidth={1}
			borderRadius={TOOLTIP_RADIUS}
			padding={6}
			gap={2}
			transformTranslateZ={TOOLTIP_Z_TRANSLATE}
			zIndexOffset={TOOLTIP_Z_OFFSET}
			depthWrite={false}
			pointerEvents="listener"
		>
			<Text fontSize={9} color={currentTheme.cardTextColor}>
				{label}
			</Text>
			{docUrl && (
				<Text
					fontSize={8}
					color={LINK_COLOR}
					cursor="pointer"
					onClick={() => window.open(docUrl, "_blank")}
				>
					docs
				</Text>
			)}
		</Container>
	);
}

function DescriptionTooltip({ description }: { description: string }) {
	return (
		<Container
			positionType="absolute"
			positionTop={-50}
			positionLeft={DESCRIPTION_TOOLTIP_LEFT}
			width={DESCRIPTION_TOOLTIP_WIDTH}
			backgroundColor={TOOLTIP_BG}
			borderColor={TOOLTIP_BORDER}
			borderWidth={1}
			borderRadius={TOOLTIP_RADIUS}
			padding={7}
			transformTranslateZ={TOOLTIP_Z_TRANSLATE}
			zIndexOffset={TOOLTIP_Z_OFFSET}
			depthWrite={false}
			pointerEvents="none"
		>
			<Text fontSize={8.5} lineHeight={10.5} color={currentTheme.cardTextColor}>
				{description}
			</Text>
		</Container>
	);
}

function getDescriptionPreview(description: string): {
	lines: { id: string; text: string }[];
	isTruncated: boolean;
} {
	const words = description.trim().split(/\s+/);
	const lines: string[] = [];
	let currentLine = "";
	let isTruncated = false;

	for (const word of words) {
		let remainingWord = word;

		while (remainingWord.length > 0) {
			const nextLine = currentLine
				? `${currentLine} ${remainingWord}`
				: remainingWord;

			if (nextLine.length <= DESCRIPTION_LINE_LENGTH) {
				currentLine = nextLine;
				remainingWord = "";
				continue;
			}

			if (currentLine) {
				lines.push(currentLine);
				currentLine = "";
			} else {
				lines.push(remainingWord.slice(0, DESCRIPTION_LINE_LENGTH));
				remainingWord = remainingWord.slice(DESCRIPTION_LINE_LENGTH);
			}

			if (lines.length === DESCRIPTION_MAX_LINES) {
				isTruncated = true;
				remainingWord = "";
				break;
			}
		}

		if (isTruncated) break;
	}

	if (!isTruncated && currentLine) {
		lines.push(currentLine);
	}

	if (lines.length > DESCRIPTION_MAX_LINES) {
		lines.length = DESCRIPTION_MAX_LINES;
		isTruncated = true;
	}

	if (isTruncated && lines.length > 0) {
		const lastLine = lines[lines.length - 1];
		lines[lines.length - 1] =
			lastLine.length > DESCRIPTION_LINE_LENGTH - 3
				? `${lastLine.slice(0, DESCRIPTION_LINE_LENGTH - 3).trimEnd()}...`
				: `${lastLine}...`;
	}

	const previewLines = lines.length > 0 ? lines : [description];

	return {
		lines: previewLines.map((text, lineNumber) => ({
			id: `${lineNumber + 1}-${text.length}-${text}`,
			text,
		})),
		isTruncated,
	};
}

function getDescriptionPreviewHeight(lineCount: number): number {
	return lineCount * DESCRIPTION_LINE_HEIGHT;
}

function DescriptionPreview({ description }: { description: string }) {
	const [isHovered, setIsHovered] = useState(false);
	const { lines, isTruncated } = getDescriptionPreview(description);
	const previewHeight = getDescriptionPreviewHeight(lines.length);

	return (
		<Container
			positionType="relative"
			width="100%"
			height={previewHeight}
			flexDirection="column"
			alignItems="stretch"
			justifyContent="flex-start"
			overflow="visible"
			onHoverChange={setIsHovered}
			pointerEvents="listener"
		>
			<Container
				width={DESCRIPTION_TEXT_WIDTH}
				height={previewHeight}
				flexDirection="column"
				alignItems="stretch"
				overflow="hidden"
			>
				{lines.map((line) => (
					<Container
						key={line.id}
						height={DESCRIPTION_LINE_HEIGHT}
						alignItems="flex-start"
						justifyContent="center"
						overflow="hidden"
					>
						<Text
							width="100%"
							fontSize={9.5}
							textAlign="left"
							color={currentTheme.cardTextColor}
						>
							{line.text}
						</Text>
					</Container>
				))}
			</Container>
			{isHovered && isTruncated && (
				<DescriptionTooltip description={description} />
			)}
		</Container>
	);
}

function EnumChip({
	value,
	muted = false,
}: {
	value: string;
	muted?: boolean;
}) {
	const [isHovered, setIsHovered] = useState(false);
	const iconContent = getEnumIconContent(value);
	const label = getEnumLabel(value);
	const docUrl = getEnumDocUrl(value);

	return (
		<Container
			positionType="relative"
			width={ENUM_CHIP_SIZE}
			height={ENUM_CHIP_SIZE}
			alignItems="center"
			justifyContent="center"
			backgroundColor="rgba(255,255,255,0.08)"
			borderRadius={4}
			cursor={docUrl ? "pointer" : undefined}
			onHoverChange={setIsHovered}
			onClick={docUrl ? () => window.open(docUrl, "_blank") : undefined}
		>
			{iconContent ? (
				<Svg
					content={iconContent}
					width={ENUM_ICON_SIZE}
					height={ENUM_ICON_SIZE}
					keepAspectRatio
					transformTranslateZ={2}
					zIndexOffset={20}
					depthWrite={false}
					opacity={muted ? MUTED_ENUM_ICON_OPACITY : ENUM_ICON_OPACITY}
				/>
			) : (
				<Text
					fontSize={8}
					color={muted ? LABEL_COLOR : currentTheme.cardTextColor}
				>
					{getShortEnumLabel(value)}
				</Text>
			)}
			{isHovered && <EnumTooltip label={label} docUrl={docUrl} />}
		</Container>
	);
}

function EnumMetaPill({
	prefix,
	value,
	muted = false,
	width,
}: {
	prefix: string;
	value: string;
	muted?: boolean;
	width: number;
}) {
	const [isHovered, setIsHovered] = useState(false);
	const iconContent = getEnumIconContent(value);
	const label = getMetaLabel(value);
	const tooltipLabel = getEnumLabel(value);
	const docUrl = getEnumDocUrl(value);

	return (
		<Container
			positionType="relative"
			width={width}
			height={META_PILL_HEIGHT}
			flexDirection="row"
			alignItems="center"
			backgroundColor={META_CHIP_BG}
			borderColor={META_CHIP_BORDER}
			borderWidth={1}
			borderRadius={4}
			paddingX={4}
			gap={3}
			cursor={docUrl ? "pointer" : undefined}
			onHoverChange={setIsHovered}
			onClick={docUrl ? () => window.open(docUrl, "_blank") : undefined}
		>
			<Text fontSize={6.5} color={LABEL_COLOR}>
				{prefix}
			</Text>
			{iconContent ? (
				<Svg
					content={iconContent}
					width={META_ICON_SIZE}
					height={META_ICON_SIZE}
					keepAspectRatio
					transformTranslateZ={2}
					zIndexOffset={20}
					depthWrite={false}
					opacity={muted ? MUTED_ENUM_ICON_OPACITY : ENUM_ICON_OPACITY}
				/>
			) : (
				<Text fontSize={7} color={LABEL_COLOR}>
					{getShortEnumLabel(value)}
				</Text>
			)}
			<Text
				fontSize={7.5}
				color={muted ? LABEL_COLOR : currentTheme.cardTextColor}
			>
				{label}
			</Text>
			{isHovered && (
				<EnumTooltip label={tooltipLabel} docUrl={docUrl} anchorWidth={width} />
			)}
		</Container>
	);
}

function EnumIconRow({
	values,
	muted = false,
	limit,
	gap = 4,
}: {
	values: string[];
	muted?: boolean;
	limit?: number;
	gap?: number;
}) {
	const visibleValues = [...new Set(values.filter(Boolean))].slice(0, limit);
	if (visibleValues.length === 0) return null;

	return (
		<Container flexDirection="row" alignItems="center" gap={gap}>
			{visibleValues.map((value) => (
				<EnumChip key={normalizeEnumValue(value)} value={value} muted={muted} />
			))}
		</Container>
	);
}

function RowLabel({ label, icon }: { label: string; icon: ReactNode }) {
	return (
		<Container
			width={ROW_LABEL_WIDTH}
			flexDirection="row"
			alignItems="center"
			gap={3}
		>
			{icon}
			<Text fontSize={7} color={LABEL_COLOR}>
				{label}
			</Text>
		</Container>
	);
}

function Panel({
	label,
	icon,
	height,
	children,
}: {
	label?: string;
	icon?: ReactNode;
	height: number;
	children: ReactNode;
}) {
	return (
		<Container
			flexDirection="column"
			height={height}
			backgroundColor={INSET_BG}
			borderRadius={5}
			padding={SECTION_PADDING}
			gap={3}
			overflow="visible"
		>
			{label && (
				<Container flexDirection="row" alignItems="center" gap={3}>
					{icon}
					<Text fontSize={7} color={LABEL_COLOR}>
						{label}
					</Text>
				</Container>
			)}
			{children}
		</Container>
	);
}

function DetailRow({
	label,
	icon,
	children,
}: {
	label: string;
	icon: ReactNode;
	children: ReactNode;
}) {
	return (
		<Container
			height={DETAIL_ROW_HEIGHT}
			flexDirection="row"
			alignItems="center"
			backgroundColor={INSET_BG}
			borderRadius={5}
			padding={SECTION_PADDING}
			gap={5}
			overflow="visible"
		>
			<RowLabel label={label} icon={icon} />
			<Container flexGrow={1} flexShrink={1} overflow="visible">
				{children}
			</Container>
		</Container>
	);
}

function LinkChip({ label, url }: { label: string; url: string }) {
	return (
		<Container
			height={14}
			alignItems="center"
			justifyContent="center"
			backgroundColor={LINK_CHIP_BG}
			borderColor={LINK_CHIP_BORDER}
			borderWidth={1}
			borderRadius={4}
			paddingX={5}
			cursor="pointer"
			onClick={() => window.open(url, "_blank")}
		>
			<Text fontSize={8.5} color={LINK_COLOR}>
				{label}
			</Text>
		</Container>
	);
}

function OwnerChip({ ownerTeam }: { ownerTeam: string }) {
	return (
		<Container
			width={OWNER_CHIP_WIDTH}
			height={14}
			flexDirection="row"
			alignItems="center"
			backgroundColor={OWNER_CHIP_BG}
			borderColor={OWNER_CHIP_BORDER}
			borderWidth={1}
			borderRadius={4}
			paddingX={4}
			gap={3}
			overflow="hidden"
		>
			<Users width={7} height={7} color={LABEL_COLOR} />
			<Text fontSize={8} color={currentTheme.cardTextColor}>
				{ownerTeam}
			</Text>
		</Container>
	);
}

export interface InfoCardProps {
	/** Block classification used for architecture visualization */
	classification?: string;
	/** Short description of the service */
	description?: string;
	/** Semver string e.g. "1.2.3" */
	version?: string;
	/** Primary programming language */
	language?: string;
	/** Web frameworks in use */
	frameworks?: string[];
	/** Deployment target */
	deployment?: string;
	/** Team responsible for this service */
	ownerTeam?: string;
	/** Authentication mechanism */
	authType?: string;
	/** Infrastructure as Code tooling */
	iacTool?: string;
	/** Monitoring platform */
	monitoring?: string;
	/** Distributed tracing tool */
	tracing?: string;
	/** Log aggregation tool */
	logAggregation?: string;
	/** Documentation URLs */
	docs?: string[];
	/** External links */
	links?: BlockLink[];
}

/**
 * InfoCard displays service metadata on a 3D card, matching the
 * card family (GithubCard, MetricsCard, etc.). Contains the same
 * information previously shown on the block's front face.
 */
export function InfoCard({
	classification,
	description,
	version,
	language,
	frameworks = [],
	deployment,
	ownerTeam,
	authType,
	iacTool,
	monitoring,
	tracing,
	logAggregation,
	docs = [],
	links = [],
}: InfoCardProps) {
	const c = currentTheme.cardTextColor;
	const normalizedDescription = description?.trim();
	const stackValues = [
		language,
		...frameworks.slice(0, 3),
		deployment,
		iacTool && iacTool !== "none" ? `iac_${iacTool}` : null,
	].filter((value): value is string => Boolean(value));
	const monitoringValues =
		monitoring && monitoring !== "none" ? [monitoring] : [];
	const telemetryValues = [tracing, logAggregation].filter(
		(value): value is string => Boolean(value && value !== "none"),
	);
	const authValue = authType && authType !== "none" ? `auth_${authType}` : null;
	const opsValues = [...monitoringValues, ...telemetryValues].filter(
		(value): value is string => Boolean(value),
	);
	const visibleLinks = links.filter((link) => link.httpUrl).slice(0, 1);
	const hasInfoLinks = docs.length > 0 || visibleLinks.length > 0;
	const hasAccessMeta = Boolean(classification || authValue);
	const hasInfoFooter = Boolean(ownerTeam || hasInfoLinks);
	const hasInfoPanel = Boolean(
		normalizedDescription || hasAccessMeta || hasInfoFooter,
	);
	const descriptionPreview = normalizedDescription
		? getDescriptionPreview(normalizedDescription)
		: null;
	const descriptionPreviewHeight = descriptionPreview
		? getDescriptionPreviewHeight(descriptionPreview.lines.length)
		: 0;
	const infoPanelRowCount = [
		Boolean(descriptionPreview),
		hasAccessMeta,
		hasInfoFooter,
	].filter(Boolean).length;
	const infoPanelNaturalHeight =
		SECTION_PADDING * 2 +
		descriptionPreviewHeight +
		(hasAccessMeta ? META_PILL_HEIGHT : 0) +
		(hasInfoFooter ? 14 : 0) +
		Math.max(0, infoPanelRowCount - 1) * 2;
	const infoPanelHeight = Math.min(
		INFO_PANEL_MAX_HEIGHT,
		Math.max(INFO_PANEL_MIN_HEIGHT, infoPanelNaturalHeight),
	);

	return (
		<Container
			sizeX={CARD_SIZE_X}
			sizeY={CARD_SIZE_Y}
			pixelSize={PIXEL_SIZE}
			flexDirection="column"
			padding={CARD_PADDING}
			gap={CARD_GAP}
			alignItems="stretch"
			overflow="visible"
		>
			{/* Header: version */}
			<Container
				height={HEADER_HEIGHT}
				flexDirection="row"
				justifyContent="space-between"
				alignItems="center"
				overflow="hidden"
			>
				<Container flexDirection="row" alignItems="center" gap={4} width={72}>
					<Info width={12} height={12} color={c} />
					<Text fontSize={14} color={c}>
						INFO
					</Text>
				</Container>
				<Container
					flexDirection="row"
					alignItems="center"
					justifyContent="flex-end"
					gap={5}
					overflow="hidden"
				>
					{version && (
						<Text fontSize={10} color={LABEL_COLOR}>
							v{version}
						</Text>
					)}
				</Container>
			</Container>

			{/* Description */}
			{hasInfoPanel && (
				<Panel height={infoPanelHeight}>
					<Container
						flexDirection="column"
						justifyContent="space-between"
						height="100%"
						gap={2}
						overflow="visible"
					>
						{normalizedDescription && (
							<DescriptionPreview description={normalizedDescription} />
						)}
						{hasAccessMeta && (
							<Container
								flexDirection="row"
								justifyContent="flex-start"
								alignItems="center"
								gap={5}
								overflow="visible"
							>
								{classification && (
									<EnumMetaPill
										prefix="TYPE"
										value={classification}
										width={TYPE_PILL_WIDTH}
									/>
								)}
								{authValue && (
									<EnumMetaPill
										prefix="AUTH"
										value={authValue}
										width={AUTH_PILL_WIDTH}
									/>
								)}
							</Container>
						)}
						{hasInfoFooter && (
							<Container
								flexDirection="row"
								justifyContent="space-between"
								alignItems="center"
								gap={5}
								overflow="visible"
							>
								<Container flexGrow={1} overflow="visible">
									{ownerTeam && (
										<Container overflow="hidden">
											<OwnerChip ownerTeam={ownerTeam} />
										</Container>
									)}
								</Container>
								<Container
									flexDirection="row"
									alignItems="center"
									gap={5}
									overflow="visible"
								>
									{docs.slice(0, 1).map((docUrl) => (
										<LinkChip
											key={`doc-${docUrl.substring(0, 15)}`}
											label="Docs"
											url={docUrl}
										/>
									))}
									{visibleLinks.map((link) => (
										<LinkChip
											key={`link-${link.httpUrl?.substring(0, 15)}`}
											label={link.siteName ?? "Link"}
											url={link.httpUrl as string}
										/>
									))}
								</Container>
							</Container>
						)}
					</Container>
				</Panel>
			)}

			{/* Stack: language / frameworks / deployment / IaC */}
			{stackValues.length > 0 && (
				<DetailRow
					label="STACK"
					icon={
						<Code
							width={ROW_LABEL_ICON_SIZE}
							height={ROW_LABEL_ICON_SIZE}
							color={LABEL_COLOR}
						/>
					}
				>
					<EnumIconRow values={stackValues} />
				</DetailRow>
			)}

			{/* Run: runtime environment, logging, monitoring, and telemetry */}
			{opsValues.length > 0 && (
				<DetailRow
					label="RUN"
					icon={
						<Activity
							width={ROW_LABEL_ICON_SIZE}
							height={ROW_LABEL_ICON_SIZE}
							color={LABEL_COLOR}
						/>
					}
				>
					<Container
						flexDirection="row"
						justifyContent="flex-start"
						alignItems="center"
						gap={4}
						overflow="visible"
					>
						<EnumIconRow values={opsValues} muted limit={7} />
					</Container>
				</DetailRow>
			)}
		</Container>
	);
}
