import type { VideoState } from "./types";

/** Onde o vídeo deveria estar agora, em segundos. */
export function expectedPosition(video: VideoState, now: number, duration = Infinity): number {
  const elapsed = video.playing ? Math.max(0, now - video.updatedAt) / 1000 : 0;
  const position = video.position + elapsed;
  if (!Number.isFinite(duration) || duration <= 0) return position;
  return video.looping ? position % duration : Math.min(position, duration);
}

/** Diferença tolerada antes de corrigir a posição (evita saltos visíveis). */
export const DRIFT_TOLERANCE = 0.35;

export function needsSeek(current: number, expected: number): boolean {
  return Math.abs(current - expected) > DRIFT_TOLERANCE;
}
