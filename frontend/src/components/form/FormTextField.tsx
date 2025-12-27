import type * as React from "react";

import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useFieldContext } from "./form-context";

interface FormTextFieldProps {
  type?: React.ComponentProps<typeof Input>["type"];
  placeholder?: string;
}

export function FormTextField({
  type = "text",
  placeholder,
}: FormTextFieldProps) {
  const field = useFieldContext<string>();
  const hasErrors = field.state.meta.errors.some((e) => e !== undefined);
  return (
    <Input
      id={field.name}
      name={field.name}
      type={type}
      value={(field.state.value as string) ?? ""}
      onChange={(e) => field.handleChange(e.target.value as string)}
      onBlur={field.handleBlur}
      placeholder={placeholder}
      className={cn(hasErrors && "aria-invalid border-destructive")}
      aria-invalid={hasErrors}
    />
  );
}
