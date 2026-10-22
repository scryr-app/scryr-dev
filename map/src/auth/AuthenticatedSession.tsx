import { OrganizationSwitcher, useAuth } from "@clerk/react";
import { useQueryClient } from "@tanstack/react-query";
import { Building2 } from "lucide-react";
import type { ReactNode } from "react";
import { useEffect } from "react";
import { EditorScope } from "@/graphql/useManifestEditor";
import { useTheme } from "@/theme/theme";
import { AccountButton } from "./AccountButton";
import styles from "./AuthenticatedSession.module.css";
import { useClerkTokenBridge } from "./useClerkTokenBridge";

const organizationSwitcherAppearance = (isDark: boolean) => ({
	elements: {
		organizationSwitcherTrigger: styles.organizationTrigger,
		organizationPreview: styles.organizationPreview,
		organizationPreviewAvatarBox: styles.organizationAvatar,
		organizationPreviewMainIdentifier: `${styles.organizationName} ${isDark ? styles.darkName : styles.lightName}`,
		organizationSwitcherTriggerIcon: styles.muted,
		organizationSwitcherPopoverCard: styles.organizationPopover,
	},
});

interface AuthenticatedSessionProps {
	children: ReactNode;
}

export function OrganizationAccountControls() {
	const theme = useTheme();
	return (
		<div className={styles.accountControls}>
			<OrganizationSwitcher
				hidePersonal
				afterCreateOrganizationUrl="/"
				afterSelectOrganizationUrl="/"
				appearance={organizationSwitcherAppearance(theme.isDarkDiagram)}
			/>
			<div className={styles.divider} />
			<AccountButton variant="embedded" />
		</div>
	);
}

export function AuthenticatedSession({ children }: AuthenticatedSessionProps) {
	const theme = useTheme();
	useClerkTokenBridge();
	const { isLoaded, orgId } = useAuth();
	const queryClient = useQueryClient();

	useEffect(() => {
		if (!isLoaded || !orgId) {
			return;
		}

		void queryClient.invalidateQueries({ queryKey: ["GetScryrMaps"] });
		void queryClient.invalidateQueries({ queryKey: ["GetBlocks"] });
	}, [isLoaded, orgId, queryClient]);

	if (!isLoaded) {
		return (
			<div className={styles.loadingScreen} style={{ background: theme.background }}>
				<div className={styles.spinner} />
			</div>
		);
	}

	if (!orgId) {
		return (
			<div className={styles.organizationScreen} style={{ background: theme.background }}>
				<div className={styles.accountPosition}>
					<AccountButton />
				</div>
				<div className={styles.organizationCard}>
					<div className={styles.cardHeader}>
						<div className={styles.organizationIcon}>
							<Building2 size={17} className={styles.buildingIcon} />
						</div>
						<div>
							<h1 className={styles.heading}>Choose an organization</h1>
							<p className={styles.description}>
								Scryr maps are scoped to your active Clerk organization.
							</p>
						</div>
					</div>
					<OrganizationSwitcher
						hidePersonal
						afterCreateOrganizationUrl="/"
						afterSelectOrganizationUrl="/"
						appearance={organizationSwitcherAppearance(theme.isDarkDiagram)}
					/>
				</div>
			</div>
		);
	}

	return (
		<EditorScope.Provider key={orgId} value={orgId}>
			{children}
		</EditorScope.Provider>
	);
}
