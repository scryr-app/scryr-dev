import { UserAvatar, useClerk } from "@clerk/react";
import { CreditCard, KeyRound, LogOut, Shield, UserCog } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import styles from "./AccountButton.module.css";

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
	const { openUserProfile, signOut } = useClerk();
	const [isOpen, setIsOpen] = useState(false);
	const [isSigningOut, setIsSigningOut] = useState(false);
	const [signOutError, setSignOutError] = useState(false);
	const menuId = useId();
	const containerRef = useRef<HTMLDivElement>(null);
	const buttonClassName =
		variant === "embedded" ? styles.embeddedButton : styles.standaloneButton;
	const avatarClassName =
		variant === "embedded" ? styles.embeddedAvatar : styles.standaloneAvatar;

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

	const logout = async () => {
		setIsSigningOut(true);
		setSignOutError(false);
		try {
			await signOut({ redirectUrl: "/" });
		} catch {
			setSignOutError(true);
		} finally {
			setIsSigningOut(false);
		}
	};

	return (
		<div ref={containerRef} className={styles.container}>
			<button
				type="button"
				aria-label="Account menu"
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
				<div id={menuId} role="menu" className={styles.menu}>
					{accountMenuItems.map(({ label, path, icon: Icon }) => (
						<button
							key={path}
							type="button"
							role="menuitem"
							onClick={() => {
								openProfile(path);
							}}
							className={styles.menuItem}
						>
							<Icon size={16} className={styles.menuIcon} />
							{label}
						</button>
					))}
					<div className={styles.divider} />
					<button
						type="button"
						role="menuitem"
						disabled={isSigningOut}
						onClick={() => void logout()}
						className={styles.menuItem}
					>
						<LogOut size={16} className={styles.menuIcon} />
						{isSigningOut ? "Logging out…" : "Log out"}
					</button>
					{signOutError && (
						<p role="alert" className={styles.error}>
							Unable to log out. Please try again.
						</p>
					)}
				</div>
			) : null}
		</div>
	);
}
