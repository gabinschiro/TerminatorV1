import { create } from "zustand";
import {
  getSystemInfo,
  getAuthState,
  getPlayerSkin,
  beginMsLogin,
  completeMsLogin,
  logout as tauriLogout,
  downloadAssets,
  launchGame,
  onDownloadProgress,
  type SystemInfo,
  type Account,
  type DeviceCodeInfo,
  type DownloadProgress,
  type LaunchResponse,
} from "@/lib/tauri";

interface DownloadState {
  active: boolean;
  progress: DownloadProgress | null;
}

export type MsLoginStatus = "idle" | "awaiting_device" | "polling" | "error";

interface LauncherState extends DownloadState {
  system: SystemInfo | null;
  version: string;
  ramMb: number;
  account: Account | null;
  skinUrl: string | null;
  launchError: string | null;
  launching: boolean;
  loadingSystem: boolean;
  msStatus: MsLoginStatus;
  msDevice: DeviceCodeInfo | null;
  msError: string | null;
  refreshSystem: () => Promise<void>;
  refreshAccount: () => Promise<void>;
  refreshSkin: () => Promise<void>;
  beginMsLogin: () => Promise<void>;
  completeMsLogin: () => Promise<void>;
  cancelMsLogin: () => void;
  logout: () => Promise<void>;
  installAndPlay: () => Promise<void>;
  setVersion: (version: string) => void;
  setRam: (ramMb: number) => void;
}

export const useLauncherStore = create<LauncherState>((set, get) => ({
  system: null,
  version: "1.21.4",
  ramMb: 4096,
  account: null,
  skinUrl: null,
  launchError: null,
  launching: false,
  loadingSystem: false,
  msStatus: "idle",
  msDevice: null,
  msError: null,
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
    await get().refreshSkin();
  },

  refreshSkin: async () => {
    const account = get().account;
    if (!account) {
      set({ skinUrl: null });
      return;
    }
    try {
      const skin = await getPlayerSkin(account.uuid);
      set({ skinUrl: skin.url });
    } catch {
      set({ skinUrl: null });
    }
  },

  beginMsLogin: async () => {
    set({ msStatus: "awaiting_device", msError: null });
    try {
      const device = await beginMsLogin();
      set({ msDevice: device });
    } catch (error) {
      set({ msStatus: "error", msError: String(error) });
    }
  },

  completeMsLogin: async () => {
    const device = get().msDevice;
    if (!device) return;
    set({ msStatus: "polling", msError: null });
    try {
      const account = await completeMsLogin(
        device.device_code,
        device.interval,
        device.expires_in,
      );
      set({ account, msStatus: "idle", msDevice: null });
      await get().refreshSkin();
    } catch (error) {
      set({ msStatus: "error", msError: String(error) });
    }
  },

  cancelMsLogin: () => {
    set({ msStatus: "idle", msDevice: null, msError: null });
  },

  logout: async () => {
    await tauriLogout();
    set({ account: null, skinUrl: null });
  },

  installAndPlay: async () => {
    if (get().active || get().launching) return;
    set({ launchError: null });
    try {
      set({ active: true, progress: null });

      const unlisten = await onDownloadProgress((progress) => {
        set({ progress });
        if (progress.done) {
          set({ active: false });
          unlisten();
        }
      });

      await downloadAssets();
      set({ active: false, progress: null });

      set({ launching: true });
      const response: LaunchResponse = await launchGame({
        version: get().version,
        ram_mb: get().ramMb,
      });
      set({ launching: false });
      if (response.error) {
        set({ launchError: response.error });
      }
    } catch (error) {
      set({ active: false, launching: false, launchError: String(error) });
    }
  },

  setVersion: (version) => set({ version }),
  setRam: (ramMb) => set({ ramMb }),
}));