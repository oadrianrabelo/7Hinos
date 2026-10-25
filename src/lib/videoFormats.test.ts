import { describe, expect, it } from "vitest";
import { formatWarning, playbackError } from "./videoFormats";

describe("formatos de vídeo", () => {
  it("não avisa sobre formatos que tocam", () => {
    expect(formatWarning("C:\\Vídeos\\abertura.MP4")).toBeNull();
    expect(formatWarning("/home/igreja/clipe.webm")).toBeNull();
    expect(formatWarning("sem-extensao")).toBeNull();
  });

  it("avisa sobre formatos que quase nunca tocam", () => {
    expect(formatWarning("C:\\Vídeos\\missões.AVI")).toContain(".avi");
    expect(formatWarning("/media/pendrive/culto.wmv")).toContain("MP4");
  });

  it("explica a falha conforme o formato", () => {
    expect(playbackError("Missões", "x.wmv")).toContain("não é suportado");
    expect(playbackError("Missões", "x.mp4")).toContain("movido");
  });
});
