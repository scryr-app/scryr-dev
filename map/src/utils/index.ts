import { type ClassValue, clsx } from "clsx";

export function cn(...inputs: ClassValue[]) {
	return clsx(inputs);
}

export { isDebug } from "./debug";
export { Svg } from "./svg";
export { wrapText } from "./wrapText";
