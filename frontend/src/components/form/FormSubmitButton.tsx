import { Loader2, type LucideProps } from "lucide-react";
import type { ForwardRefExoticComponent, RefAttributes } from "react";
import { Button } from "@/components/ui/button";
import { useFormContext } from "./form-context";

interface FormSubmitButtonProps {
  label: string;
  loadingLabel: string;
  icon?: ForwardRefExoticComponent<
    Omit<LucideProps, "ref"> & RefAttributes<SVGSVGElement>
  >;
  errorLabel: string;
}

export function FormSubmitButton({
  label,
  loadingLabel,
  errorLabel,
  icon: Icon,
}: FormSubmitButtonProps) {
  const form = useFormContext();
  const isSubmitting = form.state.isSubmitting;
  const hasErrors = form.state.errors.length > 0;
  return (
    <>
      <Button
        type="submit"
        disabled={isSubmitting}
        className="w-full"
        onClick={() => {
          form.handleSubmit();
        }}
      >
        {isSubmitting ? (
          <>
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            {loadingLabel}
          </>
        ) : (
          <>
            {Icon && <Icon className="mr-2 h-4 w-4" />}
            {label}
          </>
        )}
      </Button>
      {hasErrors && (
        <div className="text-sm text-destructive">{errorLabel}</div>
      )}
    </>
  );
}
