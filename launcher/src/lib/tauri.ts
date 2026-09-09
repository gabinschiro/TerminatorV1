import { invoke } from "@tauri-apps/api/core";

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

export interface DownloadProgress {
  downloaded_mb: number;
  total_mb: number;
  percent: number;
}

export function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>("get_system_info");
}

export function launchGame(request: LaunchRequest): Promise<LaunchResponse> {
  return invoke<LaunchResponse>("launch_game", { request });
}

export function downloadAssets(version: string): Promise<DownloadProgress> {
  return invoke<DownloadProgress>("download_assets", { version });
}