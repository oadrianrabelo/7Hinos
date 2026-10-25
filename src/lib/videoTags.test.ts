import { describe, expect, it } from "vitest";
import { guessTag, tagInfo } from "./videoTags";

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
});
