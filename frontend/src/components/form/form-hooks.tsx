import { createFormHook } from "@tanstack/react-form";
import { FormFieldContainer } from "./FormFieldContainer";
import { FormSubmitButton } from "./FormSubmitButton";
import { FormTextField } from "./FormTextField";
import { fieldContext, formContext } from "./form-context";

export const { useAppForm } = createFormHook({
  fieldComponents: {
    FormFieldContainer,
    FormTextField,
  },
  formComponents: {
    FormSubmitButton,
  },
  fieldContext,
  formContext,
});
