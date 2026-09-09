import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface SystemInfo {
  os_name: string;
  os_version: string;
  total_memory_gb: number;
  used_memory_gb: number;
  cpu_name: string;
  cpu_cores: number;
  disk_free_gb: number;
}

export interface LaunchRequest {
  java_path: string;
  version: string;
  ram_mb: number;
}

export interface LaunchResponse {
  pid: number | null;
  error: string | null;
}

export interface Account {
  username: string;
  uuid: string;
}

export interface DownloadProgress {
  version: string;
  downloaded_mb: number;
  total_mb: number;
  percent: number;
  done: boolean;
}

export const DOWNLOAD_PROGRESS_EVENT = "download://progress";

export function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>("get_system_info");
}

export function launchGame(request: LaunchRequest): Promise<LaunchResponse> {
  return invoke<LaunchResponse>("launch_game", { request });
}

export function downloadAssets(version: string): Promise<DownloadProgress> {
  return invoke<DownloadProgress>("download_assets", { version });
}

export function getAuthState(): Promise<Account> {
  return invoke<Account>("get_auth_state");
}

export function loginMicrosoft(deviceCode: string): Promise<Account> {
  return invoke<Account>("login_microsoft", { deviceCode });
}

export function logout(): Promise<void> {
  return invoke<void>("logout");
}

export function onDownloadProgress(
  handler: (progress: DownloadProgress) => void,
): Promise<() => void> {
  return listen<DownloadProgress>(DOWNLOAD_PROGRESS_EVENT, (event) =>
    handler(event.payload),
  );
}