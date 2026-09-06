import { UserAvatar, useClerk } from "@clerk/react";
import { CreditCard, KeyRound, Shield, UserCog } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";

type UserProfilePath = "/account" | "/security" | "/billing" | "/api-keys";

const accountMenuItems: Array<{
	label: string;
	path: UserProfilePath;
	icon: typeof UserCog;
}> = [
	{ label: "Profile settings", path: "/account", icon: UserCog },
	{ label: "Password & security", path: "/security", icon: Shield },
	{ label: "Billing", path: "/billing", icon: CreditCard },
	{ label: "API keys", path: "/api-keys", icon: KeyRound },
];

interface AccountButtonProps {
	variant?: "standalone" | "embedded";
}

export function AccountButton({ variant = "standalone" }: AccountButtonProps) {
	const { openUserProfile } = useClerk();
	const [isOpen, setIsOpen] = useState(false);
	const menuId = useId();
	const containerRef = useRef<HTMLDivElement>(null);
	const buttonClassName =
		variant === "embedded"
			? "flex items-center gap-2 rounded-full px-3 py-1.5 text-white/60 transition-colors hover:bg-white/10 hover:text-white/90 focus:outline-none focus:ring-2 focus:ring-cyan-400"
			: "rounded-xl border border-slate-700 bg-black/45 p-1.5 shadow-lg shadow-black/30 backdrop-blur-xl transition hover:border-slate-500 hover:bg-black/60 focus:outline-none focus:ring-2 focus:ring-cyan-400";
	const avatarClassName =
		variant === "embedded"
			? "h-6 w-6 ring-1 ring-white/10"
			: "h-8 w-8 ring-1 ring-white/10";

	useEffect(() => {
		if (!isOpen) {
			return;
		}

		const handlePointerDown = (event: MouseEvent) => {
			if (!containerRef.current?.contains(event.target as Node)) {
				setIsOpen(false);
			}
		};

		const handleEscape = (event: KeyboardEvent) => {
			if (event.key === "Escape") {
				setIsOpen(false);
			}
		};

		window.addEventListener("mousedown", handlePointerDown);
		window.addEventListener("keydown", handleEscape);

		return () => {
			window.removeEventListener("mousedown", handlePointerDown);
			window.removeEventListener("keydown", handleEscape);
		};
	}, [isOpen]);

	const openProfile = (startPath: UserProfilePath) => {
		setIsOpen(false);
		openUserProfile({ __experimental_startPath: startPath });
	};

	return (
		<div ref={containerRef} className="relative">
			<button
				type="button"
				aria-expanded={isOpen}
				aria-haspopup="menu"
				aria-controls={menuId}
				onClick={() => {
					setIsOpen((value) => !value);
				}}
				className={buttonClassName}
			>
				<UserAvatar
					appearance={{
						elements: {
							userAvatarBox: avatarClassName,
						},
					}}
				/>
			</button>
			{isOpen ? (
				<div
					id={menuId}
					role="menu"
					className="absolute right-0 top-[calc(100%+0.75rem)] min-w-56 rounded-2xl border border-white/10 bg-black/55 p-2 text-slate-100 shadow-2xl shadow-black/50 backdrop-blur-2xl"
				>
					{accountMenuItems.map(({ label, path, icon: Icon }) => (
						<button
							key={path}
							type="button"
							role="menuitem"
							onClick={() => {
								openProfile(path);
							}}
							className="flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left text-sm text-slate-200 transition hover:bg-white/10 hover:text-white"
						>
							<Icon size={16} className="text-slate-400" />
							{label}
						</button>
					))}
				</div>
			) : null}
		</div>
	);
}
