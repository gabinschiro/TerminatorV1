use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Account {
    pub username: String,
    pub uuid: String,
}

// Les tokens sont persistés pour le refresh silencieux et le lancement ;
// le frontend ne lit que account via get_auth_state.
#[derive(Serialize, Deserialize, Clone)]
pub struct MsSession {
    pub account: Account,
    pub minecraft_token: String,
    pub minecraft_token_expires_at: u64,
    pub ms_refresh_token: String,
}

pub struct AuthState(pub Mutex<Option<MsSession>>);

impl Default for AuthState {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

fn game_dir() -> PathBuf {
    std::env::var("TERMINATOR_GAME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".terminator")
        })
}

fn session_path() -> PathBuf {
    game_dir().join("auth.json")
}

pub fn save_session(session: &MsSession) {
    if let Some(parent) = session_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string(session).unwrap_or_default();
    let _ = std::fs::write(session_path(), json);
}

pub fn load_session() -> Option<MsSession> {
    let path = session_path();
    if !path.exists() {
        return None;
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
}

fn delete_session() {
    let _ = std::fs::remove_file(session_path());
}

#[tauri::command]
pub fn get_auth_state(state: State<AuthState>) -> Account {
    let mut guard = state.0.lock().unwrap();
    if guard.is_none() {
        // Restaure la session persistée au démarrage : pas de reconnexion requise.
        *guard = load_session();
    }
    guard
        .as_ref()
        .map(|s| s.account.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn logout(state: State<AuthState>) {
    *state.0.lock().unwrap() = None;
    delete_session();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_no_account() {
        let state = AuthState::default();
        assert!(state.0.lock().unwrap().is_none());
    }

    #[test]
    fn session_roundtrips_through_disk() {
        let dir = std::env::temp_dir().join(format!("terminator-auth-test-{}", std::process::id()));
        std::env::set_var("TERMINATOR_GAME_DIR", &dir);
        let session = MsSession {
            account: Account {
                username: "zqodev".into(),
                uuid: "uuid".into(),
            },
            minecraft_token: "mc-token".into(),
            minecraft_token_expires_at: 0,
            ms_refresh_token: "refresh".into(),
        };
        save_session(&session);
        let loaded = load_session().expect("should load");
        assert_eq!(loaded.account.username, "zqodev");
        assert_eq!(loaded.ms_refresh_token, "refresh");
        let _ = std::fs::remove_dir_all(&dir);
        std::env::remove_var("TERMINATOR_GAME_DIR");
    }
}