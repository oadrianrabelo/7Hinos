import { describe, expect, it } from "vitest";
import { guessTag, itemTag, resolveVideo, tagInfo } from "./videoTags";

describe("tipos de vídeo", () => {
  it("reconhece os vídeos semanais pelo nome do arquivo", () => {
    expect(guessTag("C:\\Vídeos\\Informativo Mundial das Missões 25-10.mp4")).toBe("missoes");
    expect(guessTag("/home/igreja/MISSÕES 4º tri.mp4")).toBe("missoes");
    expect(guessTag("D:\\Provai e Vede - Dízimos.mp4")).toBe("provai");
    expect(guessTag("C:\\Vídeos\\abertura congresso.mp4")).toBeNull();
  });

  it("descreve o tipo pelo id", () => {
    expect(tagInfo("provai")?.weekly).toBe(true);
    expect(tagInfo(null)).toBeNull();
    expect(tagInfo("desconhecido")).toBeNull();
  });

  it("no culto, o vídeo semanal é sempre o de hoje", () => {
    const videos = [
      { path: "/v/informativo-antigo.mp4", title: "Antigo", tag: null },
      { path: "/v/informativo-novo.mp4", title: "Novo", tag: "missoes" },
    ];
    const slot = { kind: "video" as const, path: "/v/informativo-antigo.mp4", title: "Informativo", tag: "missoes" };
    expect(resolveVideo(slot, videos).path).toBe("/v/informativo-novo.mp4");
    expect(itemTag(slot, videos)?.id).toBe("missoes");
    const plain = { kind: "video" as const, path: "/v/informativo-novo.mp4", title: "Novo" };
    expect(resolveVideo(plain, videos).path).toBe("/v/informativo-novo.mp4");
    expect(itemTag(plain, videos)?.id).toBe("missoes");
  });
});
