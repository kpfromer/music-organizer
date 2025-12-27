import { Field, FieldDescription, FieldError, FieldLabel } from "../ui/field";
import { useFieldContext } from "./form-context";

interface FormFieldContainerProps {
	label: string;
	description?: string;
	children: React.ReactNode;
}

export function FormFieldContainer({
	label,
	description,
	children,
}: FormFieldContainerProps) {
	const field = useFieldContext();
	const firstError = field.state.meta.errors.find((e) => e !== undefined);

	return (
		<Field>
			<FieldLabel htmlFor={field.name}>{label}</FieldLabel>
			{children}
			{description && <FieldDescription>{description}</FieldDescription>}
			{firstError !== undefined && <FieldError>{firstError}</FieldError>}
		</Field>
	);
}
