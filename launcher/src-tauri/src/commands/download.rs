use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

const PROGRESS_EVENT: &str = "download://progress";
const DEFAULT_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/gabinschiro/TerminatorV1/main/cdn/latest.json";
const CHUNK_COUNT: u64 = 4;

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub version: String,
    pub downloaded_mb: u64,
    pub total_mb: u64,
    pub percent: u8,
    pub done: bool,
}

#[derive(Deserialize)]
struct ClientFile {
    url: String,
    sha256: String,
}

#[derive(Deserialize)]
struct Manifest {
    version: String,
    client: ClientFile,
}

fn manifest_url() -> String {
    std::env::var("TERMINATOR_CDN_URL").unwrap_or_else(|_| DEFAULT_MANIFEST_URL.to_string())
}

fn game_dir() -> std::path::PathBuf {
    std::env::var("TERMINATOR_GAME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            std::path::PathBuf::from(home).join(".terminator")
        })
}

fn client_jar_path(version: &str) -> std::path::PathBuf {
    game_dir().join("client").join(version).join("terminator-client.jar")
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .expect("reqwest client construction")
}

// Découpe la taille totale en plages [start, end] pour un téléchargement parallèle.
fn chunk_ranges(total: u64, chunks: u64) -> Vec<(u64, u64)> {
    if total == 0 || chunks == 0 {
        return Vec::new();
    }
    let per = total / chunks;
    if per == 0 {
        return vec![(0, total.saturating_sub(1))];
    }
    (0..chunks)
        .map(|i| {
            let start = i * per;
            let end = if i == chunks - 1 { total - 1 } else { start + per - 1 };
            (start, end)
        })
        .collect()
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

async fn fetch_manifest(client: &reqwest::Client, url: &str) -> Result<Manifest, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Impossible de contacter le CDN : {e}"))?;
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    if status != 200 {
        return Err(format!(
            "Le CDN a refusé le manifest (HTTP {status}). Vérifiez TERMINATOR_CDN_URL. {body}"
        ));
    }
    serde_json::from_str(&body).map_err(|e| format!("Manifest invalide : {e}"))
}

async fn download_range(
    client: &reqwest::Client,
    url: &str,
    start: u64,
    end: u64,
) -> Result<Vec<u8>, String> {
    let resp = client
        .get(url)
        .header("Range", format!("bytes={start}-{end}"))
        .send()
        .await
        .map_err(|e| format!("Erreur réseau pendant le téléchargement : {e}"))?;
    let status = resp.status().as_u16();
    // 206 = Partial Content attendu pour une requête Range.
    if status != 206 && status != 200 {
        return Err(format!("Échec du téléchargement (HTTP {status})."));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Lecture du flux impossible : {e}"))
}

#[tauri::command]
pub async fn download_assets(app: AppHandle) -> Result<DownloadProgress, String> {
    let client = http_client();
    let manifest = fetch_manifest(&client, &manifest_url()).await?;
    let version = manifest.version.clone();
    let url = manifest.client.url.clone();
    let expected_sha = manifest.client.sha256.clone();

    let jar_path = client_jar_path(&version);
    if jar_path.exists() {
        if let Ok(bytes) = std::fs::read(&jar_path) {
            if expected_sha.is_empty() || hex_sha256(&bytes) == expected_sha {
                return Ok(DownloadProgress {
                    version,
                    downloaded_mb: bytes.len() as u64 / 1024 / 1024,
                    total_mb: bytes.len() as u64 / 1024 / 1024,
                    percent: 100,
                    done: true,
                });
            }
        }
        // Jar présent mais invalide : on le re-télécharge.
        let _ = std::fs::remove_file(&jar_path);
    }

    // Taille totale via l'en-tête Content-Length (requête HEAD, fallback GET range 0-0).
    let head = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Impossible d'interroger le CDN : {e}"))?;
    let total = head
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .ok_or_else(|| "Le CDN ne fournit pas la taille du fichier.".to_string())?;

    let ranges = chunk_ranges(total, CHUNK_COUNT);
    let mut handles = Vec::new();
    let downloaded = Arc::new(Mutex::new(0u64));

    for (start, end) in ranges {
        let client = client.clone();
        let url = url.clone();
        let downloaded = downloaded.clone();
        handles.push(tokio::spawn(async move {
            let bytes = download_range(&client, &url, start, end).await?;
            let mut guard = downloaded.lock().await;
            *guard += bytes.len() as u64;
            drop(guard);
            Ok::<Vec<u8>, String>(bytes)
        }));
    }

    let mut collected: Vec<Vec<u8>> = Vec::with_capacity(CHUNK_COUNT as usize);
    for handle in handles {
        let chunk = handle
            .await
            .map_err(|e| format!("Tâche de téléchargement interrompue : {e}"))?
            .map_err(|e: String| e)?;
        let guard = downloaded.lock().await;
        let current = *guard;
        drop(guard);
        emit_progress(&app, &version, current, total);
        collected.push(chunk);
    }

    let data: Vec<u8> = collected.into_iter().flatten().collect();

    if !expected_sha.is_empty() {
        let actual = hex_sha256(&data);
        if actual != expected_sha {
            return Err(format!(
                "Intégrité du client invalide (sha256 attendu {expected_sha}, obtenu {actual})."
            ));
        }
    }

    if let Some(parent) = jar_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Impossible de créer le dossier d'installation : {e}"))?;
    }
    tokio::fs::write(&jar_path, &data)
        .await
        .map_err(|e| format!("Impossible d'écrire le client : {e}"))?;

    Ok(DownloadProgress {
        version,
        downloaded_mb: total / 1024 / 1024,
        total_mb: total / 1024 / 1024,
        percent: 100,
        done: true,
    })
}

fn emit_progress(app: &AppHandle, version: &str, downloaded: u64, total: u64) {
    let _ = app.emit(
        PROGRESS_EVENT,
        DownloadProgress {
            version: version.to_string(),
            downloaded_mb: downloaded / 1024 / 1024,
            total_mb: total / 1024 / 1024,
            percent: if total == 0 {
                0
            } else {
                ((downloaded as f64 / total as f64) * 100.0).round() as u8
            },
            done: downloaded >= total,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_cover_whole_file() {
        let total = 1000;
        let ranges = chunk_ranges(total, 4);
        assert_eq!(ranges.len(), 4);
        assert_eq!(ranges[0].0, 0);
        assert_eq!(ranges[3].1, 999);
        // Aucun trou entre les plages.
        for pair in ranges.windows(2) {
            assert_eq!(pair[1].0, pair[0].1 + 1);
        }
    }

    #[test]
    fn ranges_handle_small_file() {
        let ranges = chunk_ranges(2, 4);
        assert_eq!(ranges, vec![(0, 1)]);
    }

    #[test]
    fn sha256_hex_is_expected() {
        assert_eq!(
            hex_sha256(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}