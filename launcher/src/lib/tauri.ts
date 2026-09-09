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

export interface PlayerSkin {
  url: string;
}

export interface DeviceCodeInfo {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
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
  return invoke<DownloadProgress>("install_client", { version });
}

export function getAuthState(): Promise<Account> {
  return invoke<Account>("get_auth_state");
}

export function getPlayerSkin(uuid: string): Promise<PlayerSkin> {
  return invoke<PlayerSkin>("get_player_skin", { uuid });
}

export function beginMsLogin(): Promise<DeviceCodeInfo> {
  return invoke<DeviceCodeInfo>("begin_ms_login");
}

export function completeMsLogin(
  deviceCode: string,
  intervalSecs: number,
  expiresIn: number,
): Promise<Account> {
  return invoke<Account>("complete_ms_login", {
    deviceCode,
    intervalSecs,
    expiresIn,
  });
}

export function refreshMsLogin(): Promise<Account> {
  return invoke<Account>("refresh_ms_login");
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