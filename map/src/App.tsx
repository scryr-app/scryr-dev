import { Show } from "@clerk/react";
import {
	AuthenticatedSession,
	isLocalAuthMode,
	OrganizationAccountControls,
	SignedOutScreen,
} from "@/auth";
import { OpenSourcePrompt } from "@/components/OpenSourcePrompt";
import { MapSceneShell } from "@/scene";

function App() {
	if (isLocalAuthMode) {
		return <MapSceneShell />;
	}

	return (
		<>
			<Show when="signed-out">
				<SignedOutScreen />
			</Show>
			<Show when="signed-in">
				<AuthenticatedSession>
					<MapSceneShell
						header={
							<div className="absolute right-4 top-4 z-50">
								<OrganizationAccountControls />
							</div>
						}
					/>
					<OpenSourcePrompt />
				</AuthenticatedSession>
			</Show>
		</>
	);
}

export default App;
