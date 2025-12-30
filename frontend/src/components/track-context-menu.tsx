import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { graphql } from "@/graphql";
import { execute } from "@/lib/execute-graphql";

const PlaylistsQuery = graphql(`
  query PlaylistsForMenu {
    playlists(page: 1, pageSize: 100) {
      playlists {
        id
        name
      }
    }
  }
`);

const CreatePlaylistMutation = graphql(`
  mutation CreatePlaylist($name: String!, $description: String) {
    createPlaylist(name: $name, description: $description) {
      id
      name
      description
      createdAt
      updatedAt
      trackCount
    }
  }
`);

const AddTrackToPlaylistMutation = graphql(`
  mutation AddTrackToPlaylist($playlistId: Int!, $trackId: Int!) {
    addTrackToPlaylist(playlistId: $playlistId, trackId: $trackId)
  }
`);

interface TrackContextMenuProps {
  trackId: number;
  trackTitle: string;
  children: React.ReactNode;
}

export function TrackContextMenu({
  trackId,
  trackTitle,
  children,
}: TrackContextMenuProps) {
  const queryClient = useQueryClient();

  const { data: playlistsData } = useQuery({
    queryKey: ["playlists-menu"],
    queryFn: async () => {
      const result = await execute(PlaylistsQuery);
      return result.playlists.playlists;
    },
  });

  const createPlaylistMutation = useMutation({
    mutationFn: async (name: string) => {
      const result = await execute(CreatePlaylistMutation, {
        name,
        description: null,
      });
      // Add track to the newly created playlist
      await execute(AddTrackToPlaylistMutation, {
        playlistId: result.createPlaylist.id,
        trackId,
      });
    },
    onSuccess: () => {
      // Invalidate queries to refresh data
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
      queryClient.invalidateQueries({ queryKey: ["playlists-menu"] });
    },
  });

  const addToPlaylistMutation = useMutation({
    mutationFn: async (playlistId: number) => {
      return await execute(AddTrackToPlaylistMutation, {
        playlistId,
        trackId,
      });
    },
    onSuccess: () => {
      // Invalidate queries to refresh data
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
    },
  });

  const handleCreateNewPlaylist = () => {
    createPlaylistMutation.mutate(trackTitle);
  };

  const handleAddToPlaylist = (playlistId: number) => {
    addToPlaylistMutation.mutate(playlistId);
  };

  const playlists = playlistsData ?? [];

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuLabel>Add to playlist</ContextMenuLabel>
        <ContextMenuSeparator />
        <ContextMenuItem onClick={handleCreateNewPlaylist}>
          <Plus className="mr-2 h-4 w-4" />
          Create New Playlist
        </ContextMenuItem>
        {playlists.length > 0 && (
          <>
            <ContextMenuSeparator />
            <ContextMenuSub>
              <ContextMenuSubTrigger>
                Add to existing playlist
              </ContextMenuSubTrigger>
              <ContextMenuSubContent>
                {playlists.map((playlist) => (
                  <ContextMenuItem
                    key={playlist.id}
                    onClick={() => handleAddToPlaylist(playlist.id)}
                  >
                    {playlist.name}
                  </ContextMenuItem>
                ))}
              </ContextMenuSubContent>
            </ContextMenuSub>
          </>
        )}
      </ContextMenuContent>
    </ContextMenu>
  );
}
