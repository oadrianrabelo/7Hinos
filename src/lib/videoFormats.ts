// Formatos de vídeo: o que o WebView toca e o que vale avisar antes do culto.

/** Aceitos ao escolher vídeos (os arriscados também, com aviso). */
export const VIDEO_EXTENSIONS = ["mp4", "m4v", "mov", "webm", "mkv", "ogv", "avi", "wmv", "mpg", "mpeg", "flv", "3gp", "vob", "ts", "mts"];

/** Quase nunca tocam no WebView, em nenhum sistema. */
const RISKY = new Set(["avi", "wmv", "mpg", "mpeg", "flv", "3gp", "vob", "ts", "mts"]);

const CONVERT = "Converta para MP4 (H.264) — o HandBrake, gratuito, faz isso em poucos cliques.";

function extension(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? "";
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

/** Aviso para um arquivo que provavelmente não vai tocar, ou `null`. */
export function formatWarning(path: string): string | null {
  const ext = extension(path);
  if (!RISKY.has(ext)) return null;
  return `Vídeos .${ext} costumam não tocar no 7Hinos. Teste antes do culto. ${CONVERT}`;
}

/** Mensagem para um vídeo que falhou ao tocar no telão. */
export function playbackError(title: string, path: string): string {
  const ext = extension(path);
  const hint = RISKY.has(ext)
    ? `O formato .${ext} não é suportado.`
    : "O arquivo pode ter sido movido, ou usa um codificador que este computador não tem.";
  return `Não deu para tocar “${title}”. ${hint} ${CONVERT}`;
}
