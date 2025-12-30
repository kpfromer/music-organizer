import { Pause, Play, Volume2, VolumeX } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/components/ui/button";
import { useAudioPlayerStore } from "@/stores/audio-player-store";

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || Number.isNaN(seconds)) return "0:00";
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export function AudioPlayer() {
  const currentTrack = useAudioPlayerStore((state) => state.currentTrack);
  const isPlaying = useAudioPlayerStore((state) => state.isPlaying);
  const volume = useAudioPlayerStore((state) => state.volume);
  const currentTime = useAudioPlayerStore((state) => state.currentTime);
  const duration = useAudioPlayerStore((state) => state.duration);
  const togglePlayPause = useAudioPlayerStore((state) => state.togglePlayPause);
  const setVolume = useAudioPlayerStore((state) => state.setVolume);
  const seekTo = useAudioPlayerStore((state) => state.seekTo);
  const initializeAudio = useAudioPlayerStore((state) => state.initializeAudio);

  useEffect(() => {
    const cleanup = initializeAudio();
    return cleanup;
  }, [initializeAudio]);

  if (!currentTrack) {
    return null;
  }

  const primaryArtist = currentTrack.artists[0]?.name ?? "Unknown Artist";
  const progressPercent = duration > 0 ? (currentTime / duration) * 100 : 0;

  const handleProgressClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percent = x / rect.width;
    const newTime = percent * duration;
    seekTo(newTime);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newVolume = parseFloat(e.target.value);
    setVolume(newVolume);
  };

  return (
    <div className="fixed bottom-0 left-0 right-0 z-50 border-t bg-background">
      <div className="container mx-auto flex h-20 items-center gap-4 px-4">
        {/* Left: Track Info */}
        <div className="flex min-w-[300px] items-center gap-3">
          {currentTrack.album.artworkUrl ? (
            <img
              src={currentTrack.album.artworkUrl}
              alt={currentTrack.album.title}
              className="h-14 w-14 rounded object-cover"
            />
          ) : (
            <div className="flex h-14 w-14 items-center justify-center rounded bg-muted text-xs text-muted-foreground">
              {currentTrack.album.title.charAt(0).toUpperCase()}
            </div>
          )}
          <div className="flex min-w-0 flex-col">
            <div className="truncate font-medium">{currentTrack.title}</div>
            <div className="truncate text-sm text-muted-foreground">
              {primaryArtist}
            </div>
          </div>
        </div>

        {/* Center: Playback Controls */}
        <div className="flex flex-1 flex-col items-center gap-2">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={togglePlayPause}
              className="h-10 w-10"
            >
              {isPlaying ? (
                <Pause className="h-5 w-5" />
              ) : (
                <Play className="h-5 w-5" />
              )}
            </Button>
          </div>
          <div className="flex w-full max-w-md items-center gap-2">
            <span className="text-xs text-muted-foreground tabular-nums">
              {formatTime(currentTime)}
            </span>
            <button
              type="button"
              className="relative h-1 flex-1 cursor-pointer rounded-full bg-muted border-0 p-0"
              onClick={handleProgressClick}
              aria-label="Seek audio"
            >
              <div
                className="h-full rounded-full bg-primary transition-all"
                style={{ width: `${progressPercent}%` }}
              />
            </button>
            <span className="text-xs text-muted-foreground tabular-nums">
              {formatTime(duration)}
            </span>
          </div>
        </div>

        {/* Right: Volume Control */}
        <div className="flex min-w-[200px] items-center gap-2">
          {volume === 0 ? (
            <VolumeX className="h-5 w-5 text-muted-foreground" />
          ) : (
            <Volume2 className="h-5 w-5 text-muted-foreground" />
          )}
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={volume}
            onChange={handleVolumeChange}
            className="h-1 flex-1 cursor-pointer appearance-none rounded-full bg-muted accent-primary"
            style={{
              background: `linear-gradient(to right, hsl(var(--primary)) 0%, hsl(var(--primary)) ${volume * 100}%, hsl(var(--muted)) ${volume * 100}%, hsl(var(--muted)) 100%)`,
            }}
          />
        </div>
      </div>
    </div>
  );
}
