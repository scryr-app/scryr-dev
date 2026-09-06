import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export { isDebug } from "./debug";
export { Svg } from "./svg";
export { wrapText } from "./wrapText";
