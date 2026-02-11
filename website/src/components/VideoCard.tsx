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
    <div className="group rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] overflow-hidden hover:border-[var(--color-primary)]/50 hover:shadow-xl transition-all">
      <div className="relative aspect-video bg-slate-950 cursor-pointer" onClick={togglePlay}>
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
          <div className="absolute inset-0 flex items-center justify-center bg-black/30 transition-opacity group-hover:bg-black/20">
            <div className="w-16 h-16 rounded-full bg-white/90 flex items-center justify-center shadow-lg group-hover:scale-110 transition-transform">
              <svg className="w-7 h-7 text-slate-900 ml-1" fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z" />
              </svg>
            </div>
          </div>
        )}
        {/* Duration badge */}
        <div className="absolute bottom-2 right-2 px-2 py-0.5 rounded bg-black/70 text-white text-xs font-mono">
          {duration}
        </div>
      </div>
      <div className="p-4">
        <h3 className="text-lg font-semibold mb-1 group-hover:text-[var(--color-primary)] transition-colors">
          {title}
        </h3>
        <p className="text-sm text-[var(--color-text-secondary)]">{description}</p>
      </div>
    </div>
  );
}
