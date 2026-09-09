import { create } from "zustand";
import {
  getSystemInfo,
  getAuthState,
  loginMicrosoft,
  logout as tauriLogout,
  downloadAssets,
  onDownloadProgress,
  type SystemInfo,
  type Account,
  type DownloadProgress,
} from "@/lib/tauri";

interface DownloadState {
  active: boolean;
  progress: DownloadProgress | null;
}

interface LauncherState extends DownloadState {
  system: SystemInfo | null;
  version: string;
  ramMb: number;
  account: Account | null;
  loadingSystem: boolean;
  refreshSystem: () => Promise<void>;
  refreshAccount: () => Promise<void>;
  login: (deviceCode: string) => Promise<void>;
  logout: () => Promise<void>;
  startDownload: () => Promise<void>;
  setVersion: (version: string) => void;
  setRam: (ramMb: number) => void;
}

export const useLauncherStore = create<LauncherState>((set, get) => ({
  system: null,
  version: "1.21.4",
  ramMb: 4096,
  account: null,
  loadingSystem: false,
  active: false,
  progress: null,

  refreshSystem: async () => {
    set({ loadingSystem: true });
    const system = await getSystemInfo();
    set({ system, loadingSystem: false });
  },

  refreshAccount: async () => {
    const account = await getAuthState();
    set({ account: account.username ? account : null });
  },

  login: async (deviceCode) => {
    const account = await loginMicrosoft(deviceCode);
    set({ account });
  },

  logout: async () => {
    await tauriLogout();
    set({ account: null });
  },

  startDownload: async () => {
    if (get().active) return;
    set({ active: true, progress: null });

    const unlisten = await onDownloadProgress((progress) => {
      set({ progress });
      if (progress.done) {
        set({ active: false });
        unlisten();
      }
    });

    await downloadAssets(get().version);
  },

  setVersion: (version) => set({ version }),
  setRam: (ramMb) => set({ ramMb }),
}));