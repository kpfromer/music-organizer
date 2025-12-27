import * as LabelPrimitive from "@radix-ui/react-label";
import type * as React from "react";

import { cn } from "@/lib/utils";

interface FormFieldProps {
	field: any;
	label: string;
	children: (field: any) => React.ReactNode;
	className?: string;
}

export function FormField({
	field,
	label,
	children,
	className,
}: FormFieldProps) {
	const hasErrors = field.state.meta.errors.some((e) => e !== undefined);
	const firstError = field.state.meta.errors.find((e) => e !== undefined);

	return (
		<div className={cn("space-y-2", className)}>
			<LabelPrimitive.Root
				htmlFor={field.name}
				className={cn(
					"flex items-center gap-2 text-sm leading-none font-medium select-none",
					hasErrors && "text-destructive",
				)}
			>
				{label}
			</LabelPrimitive.Root>
			{children(field)}
			{hasErrors && firstError && (
				<p className="text-sm text-destructive">{firstError}</p>
			)}
		</div>
	);
}
