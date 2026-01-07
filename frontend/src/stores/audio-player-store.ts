import { create } from "zustand";
import { getUrl } from "@/lib/get-url";

export interface Track {
  id: number;
  title: string;
  trackNumber?: number | null;
  duration?: number | null;
  createdAt: Date;
  album: {
    id: number;
    title: string;
    year?: number | null;
    artworkUrl?: string | null;
  };
  artists: Array<{
    id: number;
    name: string;
  }>;
}

interface AudioPlayerState {
  currentTrack: Track | null;
  isPlaying: boolean;
  volume: number;
  currentTime: number;
  duration: number;
  audioElement: HTMLAudioElement | null;
}

interface AudioPlayerActions {
  playTrack: (track: Track) => void;
  pause: () => void;
  resume: () => void;
  togglePlayPause: () => void;
  setVolume: (volume: number) => void;
  seekTo: (time: number) => void;
  setCurrentTime: (time: number) => void;
  setIsPlaying: (isPlaying: boolean) => void;
  initializeAudio: () => () => void;
}

const VOLUME_STORAGE_KEY = "audio-player-volume";

export const useAudioPlayerStore = create<
  AudioPlayerState & AudioPlayerActions
>()((set, get) => {
  // Initialize audio element
  let audioElement: HTMLAudioElement | null = null;

  const initializeAudio = () => {
    if (audioElement) return () => {};

    const audio = new Audio();
    audio.preload = "metadata";
    audioElement = audio;

    // Load volume from localStorage
    const savedVolume = localStorage.getItem(VOLUME_STORAGE_KEY);
    if (savedVolume) {
      audio.volume = parseFloat(savedVolume);
      set({ volume: parseFloat(savedVolume) });
    }

    // Event handlers
    const handleTimeUpdate = () => {
      set({ currentTime: audio.currentTime });
    };

    const handleLoadedMetadata = () => {
      set({ duration: audio.duration });
    };

    const handleEnded = () => {
      set({ isPlaying: false, currentTime: 0 });
    };

    const handleError = () => {
      set({ isPlaying: false });
      console.error("Error loading audio");
    };

    audio.addEventListener("timeupdate", handleTimeUpdate);
    audio.addEventListener("loadedmetadata", handleLoadedMetadata);
    audio.addEventListener("ended", handleEnded);
    audio.addEventListener("error", handleError);

    set({ audioElement: audio });
    // This is the cleanup function
    return () => {
      audio.removeEventListener("timeupdate", handleTimeUpdate);
      audio.removeEventListener("loadedmetadata", handleLoadedMetadata);
      audio.removeEventListener("ended", handleEnded);
      audio.removeEventListener("error", handleError);
      audio.pause();
      audio.src = "";
    };
  };

  return {
    currentTrack: null,
    isPlaying: false,
    volume: (() => {
      const saved = localStorage.getItem(VOLUME_STORAGE_KEY);
      return saved ? parseFloat(saved) : 1.0;
    })(),
    currentTime: 0,
    duration: 0,
    audioElement: null,

    initializeAudio,

    playTrack: (track: Track) => {
      const state = get();
      if (!state.audioElement) {
        throw new Error("Audio element not found");
      }

      const audio = state.audioElement;
      if (!audio) {
        throw new Error("Audio element not found");
      }
      const audioUrl = getUrl(`/audio-file/${track.id}`);

      // If same track, just toggle play/pause
      if (state.currentTrack?.id === track.id) {
        if (state.isPlaying) {
          audio.pause();
          set({ isPlaying: false });
        } else {
          audio
            .play()
            .then(() => {
              set({ isPlaying: true });
            })
            .catch((error) => {
              console.error("Error playing audio:", error);
              set({ isPlaying: false });
            });
        }
        return;
      }

      // New track - load and play
      set({ currentTrack: track, duration: track.duration ?? 0 });
      audio.src = audioUrl;
      audio.currentTime = 0;
      set({ currentTime: 0 });
      audio
        .play()
        .then(() => {
          set({ isPlaying: true });
        })
        .catch((error) => {
          console.error("Error playing audio:", error);
          set({ isPlaying: false });
        });
    },

    pause: () => {
      const state = get();
      if (state.audioElement && state.isPlaying) {
        state.audioElement.pause();
        set({ isPlaying: false });
      }
    },

    resume: () => {
      const state = get();
      if (!state.audioElement || !state.currentTrack || state.isPlaying) return;

      const audio = state.audioElement;

      // Ensure audio source is set
      const expectedUrl = getUrl(`/audio-file/${state.currentTrack.id}`);
      if (!audio.src || !audio.src.includes(expectedUrl)) {
        audio.src = expectedUrl;
      }

      audio
        .play()
        .then(() => {
          set({ isPlaying: true });
        })
        .catch((error) => {
          console.error("Error resuming audio:", error);
          set({ isPlaying: false });
        });
    },

    togglePlayPause: () => {
      const state = get();
      if (state.isPlaying) {
        get().pause();
      } else {
        get().resume();
      }
    },

    setVolume: (newVolume: number) => {
      const clampedVolume = Math.max(0, Math.min(1, newVolume));
      const state = get();
      if (state.audioElement) {
        state.audioElement.volume = clampedVolume;
      }
      localStorage.setItem(VOLUME_STORAGE_KEY, clampedVolume.toString());
      set({ volume: clampedVolume });
    },

    seekTo: (time: number) => {
      const state = get();
      if (state.audioElement && state.currentTrack) {
        const clampedTime = Math.max(0, Math.min(state.duration, time));
        state.audioElement.currentTime = clampedTime;
        set({ currentTime: clampedTime });
      }
    },

    setCurrentTime: (time: number) => {
      set({ currentTime: time });
    },

    setIsPlaying: (isPlaying: boolean) => {
      set({ isPlaying });
    },
  };
});
