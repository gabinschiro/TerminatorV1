use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize, Clone, Default)]
pub struct Account {
    pub username: String,
    pub uuid: String,
}

pub struct AuthState(pub Mutex<Account>);

impl Default for AuthState {
    fn default() -> Self {
        Self(Mutex::new(Account::default()))
    }
}

fn is_valid_device_code(device_code: &str) -> bool {
    !device_code.trim().is_empty()
}

#[tauri::command]
pub fn get_auth_state(state: State<AuthState>) -> Account {
    // Clonage immuable pour ne pas exposer le token au frontend.
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn login_microsoft(state: State<AuthState>, device_code: String) -> Result<Account, String> {
    // Phase 2 : OAuth Microsoft device-flow. Le device_code est validé
    // contre le backend. Squelette : mémorise l'état pour le frontend.
    if !is_valid_device_code(&device_code) {
        return Err("Code de vérification invalide.".into());
    }

    let account = Account {
        username: "Joueur".to_string(),
        uuid: "00000000-0000-0000-0000-000000000000".to_string(),
    };

    *state.0.lock().unwrap() = account.clone();
    Ok(account)
}

#[tauri::command]
pub fn logout(state: State<AuthState>) {
    *state.0.lock().unwrap() = Account::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_device_code_is_rejected() {
        assert!(!is_valid_device_code(""));
        assert!(!is_valid_device_code("   "));
    }

    #[test]
    fn non_empty_device_code_is_accepted() {
        assert!(is_valid_device_code("code-123"));
    }

    #[test]
    fn auth_state_roundtrip() {
        let state = AuthState::default();
        {
            let mut guard = state.0.lock().unwrap();
            *guard = Account {
                username: "Joueur".into(),
                uuid: "uuid".into(),
            };
        }
        let account = state.0.lock().unwrap();
        assert_eq!(account.username, "Joueur");
    }
}