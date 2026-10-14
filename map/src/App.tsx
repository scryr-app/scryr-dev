import { Show } from "@clerk/react";
import {
	AuthenticatedSession,
	isLocalAuthMode,
	OrganizationAccountControls,
	SignedOutScreen,
} from "@/auth";
import { OpenSourcePrompt } from "@/components/OpenSourcePrompt";
import { MapSceneShell } from "@/scene";
import styles from "./App.module.css";

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
							<div className={styles.accountControls}>
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
