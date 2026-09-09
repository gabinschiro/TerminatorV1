use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize, Clone, Default)]
pub struct Account {
    pub username: String,
    pub uuid: String,
}

// Les tokens sont persistés pour la Phase 3 (passage d'auth au client + refresh silencieux) ;
// le frontend ne lit que account via get_auth_state.
#[allow(dead_code)]
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

#[tauri::command]
pub fn get_auth_state(state: State<AuthState>) -> Account {
    state
        .0
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.account.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn logout(state: State<AuthState>) {
    *state.0.lock().unwrap() = None;
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
    fn logout_clears_session() {
        let state = AuthState::default();
        *state.0.lock().unwrap() = Some(MsSession {
            account: Account {
                username: "Joueur".into(),
                uuid: "uuid".into(),
            },
            minecraft_token: String::new(),
            minecraft_token_expires_at: 0,
            ms_refresh_token: String::new(),
        });
        *state.0.lock().unwrap() = None;
        assert!(state.0.lock().unwrap().is_none());
    }
}