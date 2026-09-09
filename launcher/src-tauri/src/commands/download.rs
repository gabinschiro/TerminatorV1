use serde::Serialize;

#[derive(Serialize)]
pub struct DownloadProgress {
    pub downloaded_mb: u64,
    pub total_mb: u64,
    pub percent: u8,
}

#[tauri::command]
pub fn download_assets(version: String) -> Result<DownloadProgress, String> {
    // Phase 3 : téléchargement parallèle depuis le CDN.
    // Le squelette expose le contrat de progression consommé par le frontend.
    let _ = version;
    Ok(DownloadProgress {
        downloaded_mb: 0,
        total_mb: 0,
        percent: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_progress_reports_zero_initially() {
        let progress = download_assets("1.21.4".to_string()).unwrap();
        assert_eq!(progress.percent, 0);
    }
}