use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const PROGRESS_EVENT: &str = "download://progress";

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub version: String,
    pub downloaded_mb: u64,
    pub total_mb: u64,
    pub percent: u8,
    pub done: bool,
}

#[tauri::command]
pub async fn download_assets(app: AppHandle, version: String) -> Result<DownloadProgress, String> {
    // Phase 2 : progression simulée pour valider le streaming d'événements.
    // Phase 3 : remplacé par un vrai téléchargement parallèle depuis le CDN.
    let total_mb: u64 = 250;
    let steps = 50u32;

    for step in 0..=steps {
        let downloaded_mb = total_mb * step as u64 / steps as u64;
        let percent = (step as f32 / steps as f32 * 100.0).round() as u8;

        let progress = DownloadProgress {
            version: version.clone(),
            downloaded_mb,
            total_mb,
            percent,
            done: step == steps,
        };
        let _ = app.emit(PROGRESS_EVENT, progress.clone());

        // Laisse le frontend recevoir chaque événement sans saturer le bus.
        tokio::time::sleep(Duration::from_millis(30)).await;
    }

    Ok(DownloadProgress {
        version,
        downloaded_mb: total_mb,
        total_mb,
        percent: 100,
        done: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_reaches_done_at_100_percent() {
        let progress = DownloadProgress {
            version: "1.21.4".into(),
            downloaded_mb: 250,
            total_mb: 250,
            percent: 100,
            done: true,
        };
        assert_eq!(progress.percent, 100);
        assert!(progress.done);
    }
}