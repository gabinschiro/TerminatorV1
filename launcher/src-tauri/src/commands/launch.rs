use crate::commands::auth::AuthState;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::State;

const KNOT_CLIENT: &str = "net.fabricmc.loader.impl.launch.knot.KnotClient";

#[derive(Deserialize)]
struct InstalledVersion {
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
}

#[derive(Deserialize)]
struct AssetIndex {
    id: String,
}

fn read_asset_index_id(version: &str) -> Option<String> {
    let path = versions_dir().join(version).join(format!("{version}.json"));
    let content = std::fs::read_to_string(path).ok()?;
    let parsed: InstalledVersion = serde_json::from_str(&content).ok()?;
    Some(parsed.asset_index.id)
}

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

fn versions_dir() -> PathBuf {
    game_dir().join("versions")
}

fn libraries_dir() -> PathBuf {
    game_dir().join("libraries")
}

fn assets_dir() -> PathBuf {
    game_dir().join("assets")
}

fn natives_dir() -> PathBuf {
    game_dir().join("natives")
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

// Classpath = jar du client + toutes les libs installées dans libraries/.
fn build_classpath(version: &str) -> Result<String, String> {
    let mut jars: Vec<PathBuf> = Vec::new();

    let client_jar = versions_dir().join(version).join(format!("{version}.jar"));
    if !client_jar.exists() {
        return Err(format!(
            "Le client {version} n'est pas installé. Lancez d'abord l'installation."
        ));
    }
    jars.push(client_jar);

    let libs = libraries_dir();
    if libs.exists() {
        let mut found = walk_jar_files(&libs);
        jars.append(&mut found);
    }

    if jars.is_empty() {
        return Err("Aucune librairie installée.".to_string());
    }

    let sep = if cfg!(windows) { ";" } else { ":" };
    Ok(jars
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(sep))
}

fn walk_jar_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "jar") {
                    out.push(path);
                }
            }
        }
    }
    out
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

    let classpath = match build_classpath(&request.version) {
        Ok(cp) => cp,
        Err(e) => {
            return LaunchResponse {
                pid: None,
                error: Some(e),
            }
        }
    };

    let game_dir = game_dir();
    let assets_index_name = read_asset_index_id(&request.version)
        .ok_or_else(|| "Version non installée ou index d'assets introuvable.".to_string());
    let assets_index_name = match assets_index_name {
        Ok(id) => id,
        Err(e) => {
            return LaunchResponse {
                pid: None,
                error: Some(e),
            }
        }
    };
    let mut cmd = Command::new(java_path());
    cmd.current_dir(&game_dir)
        .arg(format!("-Xmx{}m", request.ram_mb))
        .arg(format!("-Djava.library.path={}", natives_dir().display()))
        .arg("-cp")
        .arg(&classpath)
        .arg(KNOT_CLIENT)
        .arg("--username")
        .arg(&session.account.username)
        .arg("--version")
        .arg(&request.version)
        .arg("--gameDir")
        .arg(&game_dir)
        .arg("--assetsDir")
        .arg(assets_dir())
        .arg("--assetIndex")
        .arg(assets_index_name)
        .arg("--uuid")
        .arg(uuid_dashed(&session.account.uuid))
        .arg("--accessToken")
        .arg(&session.minecraft_token)
        .arg("--userType")
        .arg("msa")
        .arg("--versionType")
        .arg("release")
        .arg("--terminator-version")
        .arg(&request.version)
        .arg("--terminator-dir")
        .arg(&game_dir);

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
        let json = r#"{"version":"1.21.4","ram_mb":4096}"#;
        let req: LaunchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.version, "1.21.4");
        assert_eq!(req.ram_mb, 4096);
    }
}