import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FormFieldContainer } from "@/components/form/FormFieldContainer";
import { FormSubmitButton } from "@/components/form/FormSubmitButton";
import { FormTextField } from "@/components/form/FormTextField";
import { useAppForm } from "@/components/form/form-hooks";
import { Button } from "@/components/ui/button";
import { graphql } from "@/graphql";
import { execute } from "@/lib/execute-graphql";

const YoutubeSubscriptionsQuery = graphql(`
  query YoutubeSubscriptions {
    youtubeSubscriptions {
      id
      name
    }
  }
`);

const YoutubeAddSubscriptionMutation = graphql(`
  mutation YoutubeAddSubscription($name: String!) {
    addYoutubeSubscription(name: $name)
  }
`);

const YoutubeRemoveSubscriptionMutation = graphql(`
  mutation YoutubeRemoveSubscription($id: Int!) {
    removeYoutubeSubscription(id: $id)
  }
`);

export function YoutubeSubscriptions() {
  const queryClient = useQueryClient();
  const {
    data: subscriptionsData,
    status,
    error,
  } = useQuery({
    queryKey: ["youtube-subscriptions"],
    queryFn: async () => execute(YoutubeSubscriptionsQuery),
  });
  const addSubscriptionMutation = useMutation({
    mutationFn: async (name: string) =>
      execute(YoutubeAddSubscriptionMutation, { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["youtube-subscriptions"] });
      queryClient.invalidateQueries({ queryKey: ["youtube-videos"] });
    },
  });
  const removeSubscriptionMutation = useMutation({
    mutationFn: async (id: number) =>
      execute(YoutubeRemoveSubscriptionMutation, { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["youtube-subscriptions"] });
    },
  });

  const form = useAppForm({
    defaultValues: {
      name: "",
    },
    onSubmit: async ({ value }) => {
      await addSubscriptionMutation.mutateAsync(value.name);
    },
  });

  if (status === "pending") {
    return <div>Loading...</div>;
  }
  if (status === "error") {
    return <div>Error: {error.message}</div>;
  }
  if (!subscriptionsData) {
    return <div>No data</div>;
  }

  return (
    <div className="container mx-auto p-8 text-center relative z-10">
      <h2 className="text-2xl font-bold mb-8">Youtube Subscriptions</h2>
      <div className="flex flex-col gap-4 mb-8">
        {subscriptionsData?.youtubeSubscriptions.map((subscription) => (
          <div key={subscription.id}>
            <span>{subscription.name}</span>
            <Button
              variant="destructive"
              onClick={() => removeSubscriptionMutation.mutate(subscription.id)}
            >
              Remove
            </Button>
          </div>
        ))}
      </div>

      <h2 className="text-2xl font-bold mb-8">Add Subscription</h2>
      <form.AppForm>
        <form.AppField name="name">
          {() => (
            <FormFieldContainer label="Subscription Name">
              <FormTextField placeholder="Enter subscription name" />
            </FormFieldContainer>
          )}
        </form.AppField>
        <FormSubmitButton
          label="Add Subscription"
          loadingLabel="Adding..."
          errorLabel="Failed to add subscription"
        />
      </form.AppForm>
    </div>
  );
}
