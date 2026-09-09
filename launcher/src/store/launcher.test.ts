import { describe, it, expect } from "vitest";
import { useLauncherStore } from "./launcher";

describe("launcher store", () => {
  it("defaults to 1.21.4 with 4096 Mo RAM and idle download", () => {
    const state = useLauncherStore.getState();
    expect(state.version).toBe("1.21.4");
    expect(state.ramMb).toBe(4096);
    expect(state.active).toBe(false);
    expect(state.progress).toBeNull();
  });

  it("updates version and RAM via setters", () => {
    const { setVersion, setRam } = useLauncherStore.getState();
    setVersion("1.21.8");
    setRam(8192);
    expect(useLauncherStore.getState().version).toBe("1.21.8");
    expect(useLauncherStore.getState().ramMb).toBe(8192);
  });
});