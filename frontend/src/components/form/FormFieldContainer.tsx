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
  const errors = field.state.meta.errors.filter((e) => e !== undefined);

  return (
    <Field>
      <FieldLabel htmlFor={field.name}>{label}</FieldLabel>
      {children}
      {description && <FieldDescription>{description}</FieldDescription>}
      {errors.length > 0 && (
        <FieldError
          errors={errors.map((error) => {
            // Handle Zod error objects - extract message if it's an object
            if (
              typeof error === "object" &&
              error !== null &&
              "message" in error
            ) {
              return { message: String(error.message) };
            }
            // Handle string errors
            if (typeof error === "string") {
              return { message: error };
            }
            // Fallback: convert to string
            return { message: String(error) };
          })}
        />
      )}
    </Field>
  );
}
