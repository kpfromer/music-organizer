import type * as React from "react";

import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface FormInputProps {
  field: any;
  type?: React.ComponentProps<typeof Input>["type"];
  placeholder?: string;
  className?: string;
}

export function FormInput({
  field,
  type = "text",
  placeholder,
  className,
}: FormInputProps) {
  return (
    <Input
      id={field.name}
      name={field.name}
      type={type}
      value={(field.state.value as string) ?? ""}
      onChange={(e) => field.handleChange(e.target.value as string)}
      onBlur={field.handleBlur}
      placeholder={placeholder}
      className={cn(
        field.state.meta.errors.some((e) => e !== undefined) &&
          "aria-invalid border-destructive",
        className,
      )}
      aria-invalid={field.state.meta.errors.some((e) => e !== undefined)}
    />
  );
}
