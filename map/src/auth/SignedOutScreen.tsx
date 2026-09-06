import { SignInButton, SignUpButton } from "@clerk/react";

export function SignedOutScreen() {
	return (
		<div className="min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center px-6">
			<div className="w-full max-w-md rounded-xl border border-slate-700 bg-slate-900/80 p-6 space-y-4">
				<h1 className="text-xl font-semibold">Sign in to Scryr Map</h1>
				<p className="text-sm text-slate-300">
					Use Clerk to authenticate before accessing the map.
				</p>
				<div className="flex gap-3">
					<SignInButton>
						<button
							type="button"
							className="rounded-full border border-white/15 bg-white/8 px-4 py-2 text-[13px] font-medium text-white/70 transition-colors hover:bg-white/12 hover:text-white/95"
						>
							Sign in
						</button>
					</SignInButton>
					<SignUpButton>
						<button
							type="button"
							className="rounded-full border border-white/15 bg-white/5 px-4 py-2 text-[13px] font-medium text-white/60 transition-colors hover:bg-white/10 hover:text-white/90"
						>
							Sign up
						</button>
					</SignUpButton>
				</div>
			</div>
		</div>
	);
}
