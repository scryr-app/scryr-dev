import { useAuth } from "@clerk/react";
import { useEffect } from "react";
import { setClerkTokenGetter } from "@/graphql/client";

export function useClerkTokenBridge() {
	const { getToken } = useAuth();

	useEffect(() => {
		setClerkTokenGetter(async () => {
			return await getToken();
		});

		return () => {
			setClerkTokenGetter(null);
		};
	}, [getToken]);
}
