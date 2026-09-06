import { OrganizationSwitcher, useAuth } from "@clerk/react";
import { useQueryClient } from "@tanstack/react-query";
import { Building2 } from "lucide-react";
import type { ReactNode } from "react";
import { useEffect } from "react";
import { currentTheme } from "@/theme/theme";
import { AccountButton } from "./AccountButton";
import { useClerkTokenBridge } from "./useClerkTokenBridge";

const organizationSwitcherAppearance = {
	elements: {
		organizationSwitcherTrigger:
			"!min-h-9 !rounded-full !border-0 !bg-transparent !px-2.5 !py-1 text-slate-100 transition hover:!bg-white/10 focus:shadow-none focus:ring-2 focus:ring-cyan-400",
		organizationPreview: "gap-2",
		organizationPreviewAvatarBox: "h-6 w-6",
		organizationPreviewMainIdentifier:
			"max-w-48 truncate text-sm font-medium text-slate-100",
		organizationSwitcherTriggerIcon: "text-slate-400",
		organizationSwitcherPopoverCard:
			"border border-white/10 bg-black/75 text-slate-100 shadow-2xl shadow-black/50 backdrop-blur-2xl",
	},
};

interface AuthenticatedSessionProps {
	children: ReactNode;
}

export function OrganizationAccountControls() {
	return (
		<div className="flex items-center gap-0.5 rounded-full border border-white/15 bg-black/40 px-1.5 py-1 shadow-2xl backdrop-blur-md">
			<OrganizationSwitcher
				hidePersonal
				afterCreateOrganizationUrl="/"
				afterSelectOrganizationUrl="/"
				appearance={organizationSwitcherAppearance}
			/>
			<div className="h-6 w-px bg-white/10" />
			<AccountButton variant="embedded" />
		</div>
	);
}

export function AuthenticatedSession({ children }: AuthenticatedSessionProps) {
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
			<div
				className="grid h-screen w-screen place-items-center"
				style={{ background: currentTheme.background }}
			>
				<div className="h-9 w-9 animate-spin rounded-full border border-white/15 border-t-cyan-300" />
			</div>
		);
	}

	if (!orgId) {
		return (
			<div
				className="grid h-screen w-screen place-items-center px-6 text-slate-100"
				style={{ background: currentTheme.background }}
			>
				<div className="w-full max-w-sm rounded-xl border border-white/10 bg-black/50 p-5 shadow-2xl shadow-black/40 backdrop-blur-xl">
					<div className="mb-4 flex items-center gap-3">
						<div className="grid h-9 w-9 place-items-center rounded-lg border border-white/10 bg-white/8">
							<Building2 size={17} className="text-cyan-200" />
						</div>
						<div>
							<h1 className="text-base font-semibold text-white">
								Choose an organization
							</h1>
							<p className="text-sm text-slate-400">
								Scryr maps are scoped to your active Clerk organization.
							</p>
						</div>
					</div>
					<OrganizationSwitcher
						hidePersonal
						afterCreateOrganizationUrl="/"
						afterSelectOrganizationUrl="/"
						appearance={organizationSwitcherAppearance}
					/>
				</div>
			</div>
		);
	}

	return children;
}
