/** "19:30" (sem segundos) ou "19:30:05". */
export function clockText(now: Date, seconds = false): string {
  return now.toLocaleTimeString("pt-BR", {
    hour: "2-digit",
    minute: "2-digit",
    ...(seconds ? { second: "2-digit" } : {}),
  });
}

/** Tempo que falta: "04:59", "1:02:03" ou "00:00" quando acabou. */
export function countdownText(until: number, now: number): string {
  const total = Math.max(0, Math.ceil((until - now) / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${String(m).padStart(2, "0")}:${s}`;
}

/** Próxima ocorrência de "HH:MM" a partir de agora (hoje ou amanhã). */
export function nextTimeOfDay(hhmm: string, now = new Date()): number | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(hhmm.trim());
  if (!match) return null;
  const [h, m] = [Number(match[1]), Number(match[2])];
  if (h > 23 || m > 59) return null;
  const target = new Date(now);
  target.setHours(h, m, 0, 0);
  if (target.getTime() <= now.getTime()) target.setDate(target.getDate() + 1);
  return target.getTime();
}
