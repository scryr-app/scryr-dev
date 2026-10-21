import { cva, type VariantProps } from "class-variance-authority";
import { type ButtonHTMLAttributes, forwardRef } from "react";
import { cn } from "@/utils";
import styles from "./button.module.css";

const buttonVariants = cva(styles.button, {
	variants: {
		variant: {
			default: styles.primary,
			ghost: styles.ghost,
			outline: styles.outline,
		},
		size: {
			default: styles.medium,
			sm: styles.small,
			lg: styles.large,
			icon: styles.icon,
		},
	},
	defaultVariants: {
		variant: "default",
		size: "default",
	},
});

export interface ButtonProps
	extends ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> {}

/** Button component with variant and size support. */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
	({ className, variant, size, ...props }, ref) => {
		return (
			<button
				ref={ref}
				data-slot="button"
				className={cn(buttonVariants({ variant, size }), className)}
				{...props}
			/>
		);
	},
);

Button.displayName = "Button";
