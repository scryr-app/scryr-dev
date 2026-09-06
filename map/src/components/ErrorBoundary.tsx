import { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryProps {
	children: ReactNode;
	fallback?: ReactNode;
	name?: string;
}

interface ErrorBoundaryState {
	hasError: boolean;
	componentStack?: string;
	message?: string;
}

export class ErrorBoundary extends Component<
	ErrorBoundaryProps,
	ErrorBoundaryState
> {
	state: ErrorBoundaryState = {
		hasError: false,
	};

	static getDerivedStateFromError(): ErrorBoundaryState {
		return { hasError: true };
	}

	componentDidCatch(error: Error, errorInfo: ErrorInfo) {
		const boundaryName = this.props.name ?? "App";
		this.setState({
			componentStack: errorInfo.componentStack ?? undefined,
			message: error.message,
		});
		console.error(`${boundaryName} boundary caught an error`);
		console.error("message:", error.message);
		console.error("componentStack:", errorInfo.componentStack);
		console.error(error);
	}

	render() {
		if (this.state.hasError) {
			if (this.props.fallback) {
				return this.props.fallback;
			}

			return (
				<pre
					style={{
						whiteSpace: "pre-wrap",
						padding: "12px",
						fontSize: "12px",
						color: "#f8fafc",
						background: "rgba(15, 23, 42, 0.95)",
					}}
				>
					{`Error: ${this.state.message ?? "Unknown error"}\n${this.state.componentStack ?? ""}`}
				</pre>
			);
		}

		return this.props.children;
	}
}
