import { Text } from "@react-three/drei/core/Text";
import { useState } from "react";
import type { BlockLink } from "@/block/Block";
import { currentTheme } from "@/theme/theme";
import { wrapText } from "../utils/wrapText";

interface FrontFaceDisplayProps {
	color: string;
	width: number;
	hh: number; // half height
	hd: number; // half depth
	description?: string;
	version?: string;
	language?: string;
	frameworks?: string[];
	deployment?: string;
	ownerTeam?: string;
	authType?: string;
	monitoring?: string;
	tracing?: string;
	logAggregation?: string;
	docs?: string[];
	links?: BlockLink[];
}

interface LabelProps {
	position: [number, number, number];
	color: string;
	fontSize: number;
	anchorX: "left" | "center" | "right";
	maxWidth?: number;
	children: React.ReactNode;
}

function Label({
	position,
	color,
	fontSize,
	anchorX,
	maxWidth,
	children,
}: LabelProps) {
	return (
		<Text
			position={position}
			fontSize={fontSize}
			color={color}
			anchorX={anchorX}
			anchorY="top"
			maxWidth={maxWidth}
		>
			{children}
		</Text>
	);
}

interface VersionLabelProps {
	version: string;
	x: number;
	y: number;
	textColor: string;
}

function VersionLabel({ version, x, y, textColor }: VersionLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.06}
			anchorX="left"
		>
			v{version}
		</Label>
	);
}

interface OwnerTeamLabelProps {
	ownerTeam: string;
	x: number;
	y: number;
	textColor: string;
	maxWidth: number;
}

function OwnerTeamLabel({
	ownerTeam,
	x,
	y,
	textColor,
	maxWidth,
}: OwnerTeamLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.055}
			anchorX="left"
			maxWidth={maxWidth}
		>
			👥 {ownerTeam}
		</Label>
	);
}

interface AuthTypeLabelProps {
	authType: string;
	x: number;
	y: number;
	textColor: string;
	maxWidth: number;
}

function AuthTypeLabel({
	authType,
	x,
	y,
	textColor,
	maxWidth,
}: AuthTypeLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.055}
			anchorX="left"
			maxWidth={maxWidth}
		>
			🔐 {authType}
		</Label>
	);
}

interface LanguageLabelProps {
	language: string;
	x: number;
	y: number;
	textColor: string;
}

function LanguageLabel({ language, x, y, textColor }: LanguageLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.09}
			anchorX="center"
		>
			{language}
		</Label>
	);
}

interface FrameworksLabelProps {
	frameworks: string[];
	x: number;
	y: number;
	textColor: string;
	maxWidth: number;
}

function FrameworksLabel({
	frameworks,
	x,
	y,
	textColor,
	maxWidth,
}: FrameworksLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.055}
			anchorX="center"
			maxWidth={maxWidth}
		>
			{frameworks.slice(0, 3).join(", ")}
		</Label>
	);
}

interface DeploymentLabelProps {
	deployment: string;
	x: number;
	y: number;
	textColor: string;
}

function DeploymentLabel({
	deployment,
	x,
	y,
	textColor,
}: DeploymentLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.065}
			anchorX="center"
		>
			{deployment}
		</Label>
	);
}

interface MonitoringLabelProps {
	monitoring: string;
	x: number;
	y: number;
	textColor: string;
	maxWidth: number;
}

function MonitoringLabel({
	monitoring,
	x,
	y,
	textColor,
	maxWidth,
}: MonitoringLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.055}
			anchorX="right"
			maxWidth={maxWidth}
		>
			📊 {monitoring}
		</Label>
	);
}

interface TracingLabelProps {
	tracing: string;
	x: number;
	y: number;
	textColor: string;
	maxWidth: number;
}

function TracingLabel({
	tracing,
	x,
	y,
	textColor,
	maxWidth,
}: TracingLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.05}
			anchorX="right"
			maxWidth={maxWidth}
		>
			{tracing}
		</Label>
	);
}

interface LogAggregationLabelProps {
	logAggregation: string;
	x: number;
	y: number;
	textColor: string;
	maxWidth: number;
}

function LogAggregationLabel({
	logAggregation,
	x,
	y,
	textColor,
	maxWidth,
}: LogAggregationLabelProps) {
	return (
		<Label
			position={[x, y, 0]}
			color={textColor}
			fontSize={0.05}
			anchorX="right"
			maxWidth={maxWidth}
		>
			{logAggregation}
		</Label>
	);
}

interface DocsLabelProps {
	docs: string[];
	x: number;
	y: number;
	textColor: string;
}

function DocsLabel({ docs, x, y, textColor }: DocsLabelProps) {
	const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

	return (
		<>
			{docs.slice(0, 2).map((docUrl, i) => (
				<Text
					key={`doc-${docUrl.substring(0, 15)}`}
					position={[x, y - i * 0.1, 0.01]}
					color={textColor}
					fontSize={0.05}
					anchorX="right"
					onClick={() => window.open(docUrl, "_blank")}
					onPointerOver={() => setHoveredIndex(i)}
					onPointerOut={() => setHoveredIndex(null)}
					outlineWidth={hoveredIndex === i ? 0.008 : 0}
					outlineColor="#cccccc"
				>
					📄 Docs
				</Text>
			))}
		</>
	);
}

interface LinksLabelProps {
	links: BlockLink[];
	x: number;
	y: number;
	textColor: string;
}

function LinksLabel({ links, x, y, textColor }: LinksLabelProps) {
	const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

	return (
		<>
			{links.slice(0, 3).map((link, i) => {
				if (!link.httpUrl) return null;
				const displayText = link.siteName || "Link";
				return (
					<Text
						key={link.httpUrl}
						position={[x, y - i * 0.1, 0.01]}
						color={textColor}
						fontSize={0.05}
						anchorX="right"
						onClick={() => window.open(link.httpUrl as string, "_blank")}
						onPointerOver={() => setHoveredIndex(i)}
						onPointerOut={() => setHoveredIndex(null)}
						outlineWidth={hoveredIndex === i ? 0.008 : 0}
						outlineColor="#cccccc"
					>
						🔗 {displayText}
					</Text>
				);
			})}
		</>
	);
}

/**
 * Displays metadata information on the front face of the block.
 * Uses three-column layout with individual label components.
 */
export function FrontFaceDisplay({
	color,
	width,
	hh,
	hd,
	description,
	version,
	language,
	frameworks = [],
	deployment,
	ownerTeam,
	authType,
	monitoring,
	tracing,
	logAggregation,
	docs = [],
	links = [],
}: FrontFaceDisplayProps) {
	const textColor = currentTheme.getContrastingTextColor(color);

	// Column positions
	const leftX = -width * 0.35;
	const centerX = 0;
	const rightX = width * 0.35;
	const columnWidth = width * 0.3;

	// Description positioning (above columns)
	let descriptionHeight = 0;
	if (description) {
		const lines = wrapText(description, 35).slice(0, 3);
		descriptionHeight = lines.length * 0.08 + 0.15;
	}

	// Y offsets for each column (adjusted for description)
	const columnStartY = hh - 0.1 - descriptionHeight;
	let leftY = columnStartY;
	let centerY = columnStartY;
	let rightY = columnStartY;

	const lineSpacing = 0.09;
	const sectionSpacing = 0.02;

	// Calculate positions for each label
	const leftColumn = [];
	const centerColumn = [];
	const rightColumn = [];

	// LEFT COLUMN
	if (version) {
		leftColumn.push({ type: "version", y: leftY, data: version });
		leftY -= lineSpacing;
	}
	if (ownerTeam) {
		leftColumn.push({ type: "ownerTeam", y: leftY, data: ownerTeam });
		leftY -= lineSpacing;
	}
	if (authType) {
		leftColumn.push({ type: "authType", y: leftY, data: authType });
	}

	// CENTER COLUMN
	if (language) {
		centerColumn.push({ type: "language", y: centerY, data: language });
		centerY -= lineSpacing + 0.02;
	}
	if (frameworks.length > 0) {
		centerColumn.push({ type: "frameworks", y: centerY, data: frameworks });
		centerY -= lineSpacing;
	}
	if (deployment) {
		centerColumn.push({ type: "deployment", y: centerY, data: deployment });
	}

	// RIGHT COLUMN
	if (monitoring && monitoring !== "none") {
		rightColumn.push({ type: "monitoring", y: rightY, data: monitoring });
		rightY -= lineSpacing;
	}
	if (tracing && tracing !== "none") {
		rightColumn.push({ type: "tracing", y: rightY, data: tracing });
		rightY -= lineSpacing - 0.01;
	}
	if (logAggregation && logAggregation !== "none") {
		rightColumn.push({
			type: "logAggregation",
			y: rightY,
			data: logAggregation,
		});
		rightY -= lineSpacing;
	}
	if (docs && docs.length > 0) {
		rightColumn.push({ type: "docs", y: rightY, data: docs });
		rightY -= docs.slice(0, 2).length * 0.1 + sectionSpacing;
	}
	if (links && links.length > 0) {
		rightColumn.push({ type: "links", y: rightY, data: links });
	}

	return (
		<group position={[0, 0, hd + 0.01]} rotation={[0, 0, 0]}>
			{/* DESCRIPTION - Above columns */}
			{description &&
				wrapText(description, 35)
					.slice(0, 3)
					.map((line, i) => (
						<Label
							key={`desc-${line.substring(0, 10)}`}
							position={[-width * 0.4, hh - 0.1 - i * 0.08, 0]}
							color={textColor}
							fontSize={0.07}
							anchorX="left"
							maxWidth={width * 0.85}
						>
							{line}
						</Label>
					))}

			{/* LEFT COLUMN */}
			{leftColumn.map((item, i) => {
				const key = `left-${item.type}-${i}`;
				switch (item.type) {
					case "version":
						return (
							<VersionLabel
								key={key}
								version={item.data as string}
								x={leftX}
								y={item.y}
								textColor={textColor}
							/>
						);
					case "ownerTeam":
						return (
							<OwnerTeamLabel
								key={key}
								ownerTeam={item.data as string}
								x={leftX}
								y={item.y}
								textColor={textColor}
								maxWidth={columnWidth}
							/>
						);
					case "authType":
						return (
							<AuthTypeLabel
								key={key}
								authType={item.data as string}
								x={leftX}
								y={item.y}
								textColor={textColor}
								maxWidth={columnWidth}
							/>
						);
					default:
						return null;
				}
			})}

			{/* CENTER COLUMN */}
			{centerColumn.map((item, i) => {
				const key = `center-${item.type}-${i}`;
				switch (item.type) {
					case "language":
						return (
							<LanguageLabel
								key={key}
								language={item.data as string}
								x={centerX}
								y={item.y}
								textColor={textColor}
							/>
						);
					case "frameworks":
						return (
							<FrameworksLabel
								key={key}
								frameworks={item.data as string[]}
								x={centerX}
								y={item.y}
								textColor={textColor}
								maxWidth={columnWidth}
							/>
						);
					case "deployment":
						return (
							<DeploymentLabel
								key={key}
								deployment={item.data as string}
								x={centerX}
								y={item.y}
								textColor={textColor}
							/>
						);
					default:
						return null;
				}
			})}

			{/* RIGHT COLUMN */}
			{rightColumn.map((item, i) => {
				const key = `right-${item.type}-${i}`;
				switch (item.type) {
					case "monitoring":
						return (
							<MonitoringLabel
								key={key}
								monitoring={item.data as string}
								x={rightX}
								y={item.y}
								textColor={textColor}
								maxWidth={columnWidth}
							/>
						);
					case "tracing":
						return (
							<TracingLabel
								key={key}
								tracing={item.data as string}
								x={rightX}
								y={item.y}
								textColor={textColor}
								maxWidth={columnWidth}
							/>
						);
					case "logAggregation":
						return (
							<LogAggregationLabel
								key={key}
								logAggregation={item.data as string}
								x={rightX}
								y={item.y}
								textColor={textColor}
								maxWidth={columnWidth}
							/>
						);
					case "docs":
						return (
							<DocsLabel
								key={key}
								docs={item.data as string[]}
								x={rightX}
								y={item.y}
								textColor={textColor}
							/>
						);
					case "links":
						return (
							<LinksLabel
								key={key}
								links={item.data as BlockLink[]}
								x={rightX}
								y={item.y}
								textColor={textColor}
							/>
						);
					default:
						return null;
				}
			})}
		</group>
	);
}
