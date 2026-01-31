import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { FormFieldContainer } from "@/components/form/FormFieldContainer";
import { FormSubmitButton } from "@/components/form/FormSubmitButton";
import { FormTextField } from "@/components/form/FormTextField";
import { useAppForm } from "@/components/form/form-hooks";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { graphql } from "@/graphql";
import { execute } from "@/lib/execute-graphql";

const YoutubeVideosQuery = graphql(`
  query YoutubeVideos($watched: Boolean) {
    youtubeVideos(watched: $watched) {
        id
        title
        channelName
        publishedAt
        thumbnailUrl
        videoUrl
        watched
    }
  }
`);

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

const YoutubeMarkVideoAsWatchedMutation = graphql(`
  mutation YoutubeMarkVideoAsWatched($id: Int!) {
    markYoutubeVideoAsWatched(id: $id)
  }
`);

export function Youtube() {
  const [watched, setWatched] = useState<undefined | boolean>(false);

  const queryClient = useQueryClient();
  const { data, status, error } = useQuery({
    queryKey: ["youtube-videos", watched],
    queryFn: async () =>
      execute(YoutubeVideosQuery, watched !== undefined ? { watched } : {}),
  });
  const { data: subscriptionsData } = useQuery({
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
  const markVideoAsWatchedMutation = useMutation({
    mutationFn: async (id: number) =>
      execute(YoutubeMarkVideoAsWatchedMutation, { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["youtube-videos"] });
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
  if (!data) {
    return <div>No data</div>;
  }

  const videos = data.youtubeVideos;

  return (
    <div className="container mx-auto p-8 text-center relative z-10">
      <h1 className="text-3xl font-bold mb-8">Youtube Videos</h1>
      <h2 className="text-2xl font-bold mb-8">Subscriptions</h2>
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

      <div className="flex flex-row gap-4 mb-8">
        <Button onClick={() => setWatched(undefined)}>All</Button>
        <Button onClick={() => setWatched(false)}>Unwatched</Button>
        <Button onClick={() => setWatched(true)}>Watched</Button>
      </div>

      <h2 className="text-2xl font-bold mb-8">
        {watched === undefined ? "All" : watched ? "Watched" : "Unwatched"}{" "}
        Videos
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8 justify-items-center">
        {videos.map((video) => (
          <Card key={video.id}>
            <CardHeader className="gap-4">
              <CardTitle>{video.title}</CardTitle>
              <CardDescription>{video.channelName}</CardDescription>
            </CardHeader>
            <CardContent>
              <a
                href={video.videoUrl}
                target="_blank"
                rel="noopener noreferrer"
              >
                <img src={video.thumbnailUrl} alt={video.title} />
              </a>
              <Button
                variant="outline"
                onClick={() => markVideoAsWatchedMutation.mutate(video.id)}
              >
                {video.watched ? "Mark as unwatched" : "Mark as watched"}
              </Button>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
