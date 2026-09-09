use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize)]
pub struct LaunchRequest {
    pub java_path: String,
    pub version: String,
    pub ram_mb: u32,
}

#[derive(Serialize)]
pub struct LaunchResponse {
    pub pid: Option<u32>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn launch_game(request: LaunchRequest) -> LaunchResponse {
    let game_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let mut cmd = Command::new(&request.java_path);
    cmd.arg(format!("-Xmx{}m", request.ram_mb))
        .arg("-jar")
        .arg("terminator-client.jar")
        .arg("--terminator-version")
        .arg(&request.version)
        .arg("--terminator-dir")
        .arg(game_dir.join(".terminator"));

    match cmd.spawn() {
        Ok(child) => LaunchResponse {
            pid: Some(child.id()),
            error: None,
        },
        Err(e) => LaunchResponse {
            pid: None,
            error: Some(format!("Impossible de lancer le jeu : {e}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_request_serializes() {
        let json = r#"{"java_path":"/usr/bin/java","version":"1.21.4","ram_mb":4096}"#;
        let req: LaunchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.version, "1.21.4");
        assert_eq!(req.ram_mb, 4096);
    }
}