import { useQuery } from "@tanstack/react-query";
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
  query YoutubeVideos {
    youtubeVideos {
        id
        title
        channelName
        publishedAt
        thumbnailUrl
        videoUrl
    }
  }
`);

export function Youtube() {
  const { data, status, error } = useQuery({
    queryKey: ["youtube-videos"],
    queryFn: async () => execute(YoutubeVideosQuery),
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
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
