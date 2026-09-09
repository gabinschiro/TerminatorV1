import { create } from "zustand";
import { getSystemInfo, type SystemInfo } from "@/lib/tauri";

interface LauncherState {
  system: SystemInfo | null;
  version: string;
  ramMb: number;
  accountName: string | null;
  loadingSystem: boolean;
  refreshSystem: () => Promise<void>;
  setVersion: (version: string) => void;
  setRam: (ramMb: number) => void;
  setAccount: (name: string | null) => void;
}

export const useLauncherStore = create<LauncherState>((set) => ({
  system: null,
  version: "1.21.4",
  ramMb: 4096,
  accountName: null,
  loadingSystem: false,
  refreshSystem: async () => {
    set({ loadingSystem: true });
    const system = await getSystemInfo();
    set({ system, loadingSystem: false });
  },
  setVersion: (version) => set({ version }),
  setRam: (ramMb) => set({ ramMb }),
  setAccount: (accountName) => set({ accountName }),
}));