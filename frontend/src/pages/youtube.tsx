import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "@/components/ui/button";
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

const YoutubeMarkVideoAsWatchedMutation = graphql(`
  mutation YoutubeMarkVideoAsWatched($id: Int!) {
    markYoutubeVideoAsWatched(id: $id)
  }
`);

const YoutubeMarkVideoAsUnwatchedMutation = graphql(`
  mutation YoutubeMarkVideoAsUnwatched($id: Int!) {
    markYoutubeVideoAsUnwatched(id: $id)
  }
`);

export function Youtube() {
  const [watched, setWatched] = useState<undefined | boolean>(false);

  const queryClient = useQueryClient();
  const { data, status, error } = useQuery({
    queryKey: ["youtube-videos", watched],
    queryFn: async () => execute(YoutubeVideosQuery, { watched }),
  });
  const markVideoAsWatchedMutation = useMutation({
    mutationFn: async (id: number) =>
      execute(YoutubeMarkVideoAsWatchedMutation, { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["youtube-videos"] });
    },
  });
  const markVideoAsUnwatchedMutation = useMutation({
    mutationFn: async (id: number) =>
      execute(YoutubeMarkVideoAsUnwatchedMutation, { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["youtube-videos"] });
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
      <div className="flex flex-row gap-4 mb-8">
        <Button onClick={() => setWatched(undefined)}>All</Button>
        <Button onClick={() => setWatched(false)}>Unwatched</Button>
        <Button onClick={() => setWatched(true)}>Watched</Button>
      </div>

      <h2 className="text-2xl font-bold mb-8">
        {watched === undefined ? "All" : watched ? "Watched" : "Unwatched"}{" "}
        Videos
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8 justify-items-center">
        {videos.map((video) => (
          <div key={video.id} className="flex flex-col gap-2 items-start">
            <a
              href={video.videoUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="flex flex-col gap-2 items-start"
            >
              <img
                src={video.thumbnailUrl}
                alt={video.title}
                className="w-full h-auto rounded-md"
              />

              <span className="text-lg font-bold text-start">
                {video.title.slice(0, 50)}
              </span>
              <span className="text-sm text-muted-foreground">
                {video.channelName}
              </span>
            </a>
            <Button
              variant="outline"
              onClick={() => {
                if (video.watched) {
                  markVideoAsUnwatchedMutation.mutate(video.id);
                } else {
                  markVideoAsWatchedMutation.mutate(video.id);
                }
              }}
            >
              {video.watched ? "Mark as unwatched" : "Mark as watched"}
            </Button>
          </div>
          // <Card key={video.id}>
          //   <CardHeader className="gap-4">
          //     <CardTitle>{video.title}</CardTitle>
          //     <CardDescription>{video.channelName}</CardDescription>
          //   </CardHeader>
          //   <CardContent>
          //     <a
          //       href={video.videoUrl}
          //       target="_blank"
          //       rel="noopener noreferrer"
          //     >
          //       <img src={video.thumbnailUrl} alt={video.title} />
          //     </a>
          //     <Button
          //       variant="outline"
          //       onClick={() => {
          //         if (video.watched) {
          //           markVideoAsUnwatchedMutation.mutate(video.id);
          //         } else {
          //           markVideoAsWatchedMutation.mutate(video.id);
          //         }
          //       }}
          //     >
          //       {video.watched ? "Mark as unwatched" : "Mark as watched"}
          //     </Button>
          //   </CardContent>
          // </Card>
        ))}
      </div>
    </div>
  );
}
