"use client";

import { useRef, useState } from "react";

type VideoCardProps = {
  src: string;
  title: string;
  description: string;
  duration: string;
};

export default function VideoCard({ src, title, description, duration }: VideoCardProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);

  const togglePlay = () => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) {
      video.play();
      setIsPlaying(true);
    } else {
      video.pause();
      setIsPlaying(false);
    }
  };

  return (
    <div className="group rounded border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden glow-border glow-border-hover transition-all theme-transition">
      <div className="relative aspect-video bg-[#0d1117] cursor-pointer" onClick={togglePlay}>
        <video
          ref={videoRef}
          src={src}
          className="w-full h-full object-contain"
          loop
          muted
          playsInline
          preload="metadata"
          onPlay={() => setIsPlaying(true)}
          onPause={() => setIsPlaying(false)}
        />
        {/* Play overlay */}
        {!isPlaying && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/40 transition-opacity group-hover:bg-black/25">
            <div className="w-14 h-14 rounded-full border-2 border-[var(--color-primary)]/60 bg-black/50 flex items-center justify-center group-hover:scale-110 group-hover:border-[var(--color-primary)] transition-all">
              <svg className="w-6 h-6 text-[var(--color-primary)] ml-0.5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z" />
              </svg>
            </div>
          </div>
        )}
        {/* Duration badge */}
        <div className="absolute bottom-2 right-2 px-1.5 py-0.5 rounded bg-black/80 text-emerald-400 text-xs font-mono font-bold border border-emerald-500/20">
          {duration}
        </div>
      </div>
      <div className="p-4">
        <h3 className="text-lg font-mono font-bold mb-1 group-hover:text-[var(--color-primary)] transition-colors">
          {title}
        </h3>
        <p className="text-base text-[var(--color-text-secondary)]">{description}</p>
      </div>
    </div>
  );
}
