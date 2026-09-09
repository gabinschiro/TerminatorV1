use crate::commands::auth::AuthState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tauri::State;

#[derive(Deserialize)]
pub struct LaunchRequest {
    pub version: String,
    pub ram_mb: u32,
}

#[derive(Serialize)]
pub struct LaunchResponse {
    pub pid: Option<u32>,
    pub error: Option<String>,
}

fn game_dir() -> PathBuf {
    std::env::var("TERMINATOR_GAME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".terminator")
        })
}

fn client_jar_path(version: &str) -> PathBuf {
    game_dir().join("client").join(version).join("terminator-client.jar")
}

fn java_path() -> String {
    std::env::var("TERMINATOR_JAVA_PATH").unwrap_or_else(|_| "java".to_string())
}

fn uuid_dashed(uuid: &str) -> String {
    if uuid.len() == 32 && !uuid.contains('-') {
        format!(
            "{}-{}-{}-{}-{}",
            &uuid[0..8],
            &uuid[8..12],
            &uuid[12..16],
            &uuid[16..20],
            &uuid[20..32]
        )
    } else {
        uuid.to_string()
    }
}

#[tauri::command]
pub fn launch_game(state: State<AuthState>, request: LaunchRequest) -> LaunchResponse {
    // La session est lue côté backend : le token ne transite jamais par le frontend.
    let guard = state.0.lock().expect("auth state poisoned");
    let session = match &*guard {
        Some(s) => s,
        None => {
            return LaunchResponse {
                pid: None,
                error: Some("Connectez-vous avec un compte Microsoft avant de jouer.".into()),
            }
        }
    };

    let jar = client_jar_path(&request.version);
    if !jar.exists() {
        return LaunchResponse {
            pid: None,
            error: Some(format!(
                "Le client {} n'est pas installé. Lancez d'abord le téléchargement.",
                request.version
            )),
        };
    }

    // La session est injectée dans le processus Java : le client lit ces args
    // pour authentifier le joueur auprès des serveurs (voir TerminatorClient).
    let game_dir = game_dir();
    let mut cmd = Command::new(java_path());
    cmd.current_dir(&game_dir)
        .arg(format!("-Xmx{}m", request.ram_mb))
        .arg("-jar")
        .arg(&jar)
        .arg("--terminator-version")
        .arg(&request.version)
        .arg("--terminator-dir")
        .arg(&game_dir)
        .arg("--terminator-username")
        .arg(&session.account.username)
        .arg("--terminator-uuid")
        .arg(uuid_dashed(&session.account.uuid))
        .arg("--terminator-access-token")
        .arg(&session.minecraft_token);

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
    fn uuid_is_dashed_when_compact() {
        assert_eq!(
            uuid_dashed("f0dac655edd143a3ac51c07980a94244"),
            "f0dac655-edd1-43a3-ac51-c07980a94244"
        );
    }

    #[test]
    fn uuid_keeps_existing_dashes() {
        let uuid = "f0dac655-edd1-43a3-ac51-c07980a94244";
        assert_eq!(uuid_dashed(uuid), uuid);
    }

    #[test]
    fn launch_request_serializes() {
        let json = r#"{"version":"0.1.0","ram_mb":4096}"#;
        let req: LaunchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.version, "0.1.0");
        assert_eq!(req.ram_mb, 4096);
    }
}