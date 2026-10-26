import { describe, expect, it } from "vitest";
import type { ScreenStyle } from "./types";
import { MESSAGE_DEFAULTS, MONITOR_DEFAULTS, SCREEN_DEFAULTS, SLIDE_DEFAULTS, isDefault } from "./defaults";

describe("valores de fábrica", () => {
  it("reconhece quando nada mudou", () => {
    expect(isDefault({ ...SLIDE_DEFAULTS }, SLIDE_DEFAULTS)).toBe(true);
    expect(isDefault({ ...SCREEN_DEFAULTS }, SCREEN_DEFAULTS)).toBe(true);
  });

  it("percebe qualquer mudança", () => {
    expect(isDefault({ ...SLIDE_DEFAULTS, uppercase: false }, SLIDE_DEFAULTS)).toBe(false);
    expect(isDefault<ScreenStyle>({ ...SCREEN_DEFAULTS, backgroundImage: "logo.png" }, SCREEN_DEFAULTS)).toBe(false);
  });

  it("olha só os campos do grupo", () => {
    const monitor = { ...MONITOR_DEFAULTS, ...MESSAGE_DEFAULTS, messageColor: "#ffffff" };
    expect(isDefault(monitor, MONITOR_DEFAULTS)).toBe(true);
    expect(isDefault(monitor, MESSAGE_DEFAULTS)).toBe(false);
  });

  it("tolera arredondamento do controle deslizante", () => {
    expect(isDefault({ ...SLIDE_DEFAULTS, textScale: 1.0000001 }, SLIDE_DEFAULTS)).toBe(true);
  });
});
