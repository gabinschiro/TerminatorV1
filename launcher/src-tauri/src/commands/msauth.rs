use crate::commands::auth::{Account, AuthState, MsSession};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;

const DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_LOGIN_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

// Fallback compilé : remplacé par la variable d'environnement au lancement.
const MSA_CLIENT_ID: &str = "";

const SCOPE: &str = "XboxLive.signin offline_access";
const XBL_RP: &str = "http://auth.xboxlive.com";
const XSTS_RP: &str = "rp://api.minecraftservices.com/";

#[derive(Serialize, Clone)]
pub struct DeviceCodeInfo {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize, Debug, PartialEq)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[derive(Deserialize)]
struct PollError {
    error: String,
}

#[derive(Deserialize)]
struct XblResponse {
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XuiClaims,
}

#[derive(Deserialize)]
struct XuiClaims {
    xui: Vec<XuiEntry>,
}

#[derive(Deserialize)]
struct XuiEntry {
    uhs: String,
}

#[derive(Deserialize)]
struct MinecraftTokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

fn msa_client_id() -> String {
    std::env::var("TERMINATOR_MS_CLIENT_ID").unwrap_or_else(|_| MSA_CLIENT_ID.to_string())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("reqwest client construction")
}

#[derive(Debug, PartialEq)]
enum PollOutcome {
    Pending,
    Authorized(TokenResponse),
    Declined,
    Expired,
    SlowDown,
    Failed(String),
}

// Déduit l'état de l'OAuth device flow depuis la réponse HTTP et son corps JSON.
// Le device code flow communique via HTTP 400 + champ "error" tant que rien n'est validé.
fn classify_poll_response(status: u16, body: &str) -> PollOutcome {
    if status == 200 {
        return serde_json::from_str::<TokenResponse>(body)
            .map(PollOutcome::Authorized)
            .unwrap_or_else(|e| PollOutcome::Failed(format!("Réponse token invalide : {e}")));
    }
    let err = serde_json::from_str::<PollError>(body)
        .map(|p| p.error)
        .unwrap_or_else(|_| "unknown".to_string());
    match (status, err.as_str()) {
        (400, "authorization_pending") => PollOutcome::Pending,
        (400, "authorization_declined") => PollOutcome::Declined,
        (400, "expired_token") => PollOutcome::Expired,
        (400, "slow_down") => PollOutcome::SlowDown,
        (400, "bad_verification_code") => {
            PollOutcome::Failed("Code de vérification invalide.".to_string())
        }
        _ => PollOutcome::Failed(format!(
            "Échec de l'autorisation Microsoft (HTTP {status} : {err})."
        )),
    }
}

fn xsts_error_message(xerr: &str) -> Option<String> {
    match xerr {
        "2148916233" => Some(
            "Ce compte n'est lié à aucun profil Xbox (le compte doit posséder un gamertag)."
                .to_string(),
        ),
        "2148916235" => Some(
            "Xbox Live n'est pas disponible dans la région de ce compte.".to_string(),
        ),
        "2148916238" => Some(
            "Ce compte est un compte enfant : il nécessite l'ajout au groupe familial."
                .to_string(),
        ),
        _ => None,
    }
}

fn auth_bearer(client: &reqwest::Client, url: &str, token: &str) -> reqwest::RequestBuilder {
    client.get(url).header("Authorization", format!("Bearer {token}"))
}

#[tauri::command]
pub async fn begin_ms_login() -> Result<DeviceCodeInfo, String> {
    let client_id = msa_client_id();
    if client_id.is_empty() {
        return Err(
            "Login Microsoft non configuré. Définissez TERMINATOR_MS_CLIENT_ID \
             (voir docs/azure-setup.md)."
                .to_string(),
        );
    }

    let client = http_client();
    let resp = client
        .post(DEVICE_CODE_URL)
        .form(&[
            ("client_id", client_id.as_str()),
            ("scope", SCOPE),
        ])
        .send()
        .await
        .map_err(|e| format!("Impossible de contacter Microsoft : {e}"))?;

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    if status != 200 {
        return Err(format!(
            "Microsoft a refusé la demande de code (HTTP {status}). Vérifiez le client_id \
             (docs/azure-setup.md). Réponse : {body}"
        ));
    }

    let parsed: DeviceCodeResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Réponse device code invalide : {e}"))?;

    Ok(DeviceCodeInfo {
        device_code: parsed.device_code,
        user_code: parsed.user_code,
        verification_uri: parsed.verification_uri,
        expires_in: parsed.expires_in,
        interval: parsed.interval,
    })
}

#[tauri::command]
pub async fn complete_ms_login(
    state: State<'_, AuthState>,
    device_code: String,
    mut interval_secs: u64,
    expires_in: u64,
) -> Result<Account, String> {
    let client_id = msa_client_id();
    if client_id.is_empty() {
        return Err(
            "Login Microsoft non configuré. Définissez TERMINATOR_MS_CLIENT_ID \
             (voir docs/azure-setup.md)."
                .to_string(),
        );
    }

    let client = http_client();
    let deadline = now_unix() + expires_in;

    loop {
        if now_unix() >= deadline {
            return Err("Le code de vérification a expiré. Relancez la connexion.".to_string());
        }

        let resp = client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id.as_str()),
                ("device_code", device_code.as_str()),
            ])
            .send()
            .await
            .map_err(|e| format!("Erreur réseau pendant la vérification : {e}"))?;

        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();

        match classify_poll_response(status, &body) {
            PollOutcome::Pending => {
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
            PollOutcome::SlowDown => {
                interval_secs += 5;
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
            PollOutcome::Declined => {
                return Err("Connexion refusée dans le navigateur.".to_string());
            }
            PollOutcome::Expired => {
                return Err("Le code de vérification a expiré. Relancez la connexion.".to_string());
            }
            PollOutcome::Failed(msg) => return Err(msg),
            PollOutcome::Authorized(token) => {
                let session = exchange_tokens(&client, &token)
                    .await
                    .map_err(|e| format!("Échange des jetons échoué : {e}"))?;

                // Partage l'état de session avec le reste du launcher (get_auth_state, logout).
                let mut guard = state.0.lock().expect("auth state poisoned");
                *guard = Some(session);
                return guard
                    .as_ref()
                    .map(|s| s.account.clone())
                    .ok_or_else(|| "Session introuvable après connexion.".to_string());
            }
        }
    }
}

async fn exchange_tokens(client: &reqwest::Client, ms_token: &TokenResponse) -> Result<MsSession, String> {
    let (xbl_token, user_hash) = authenticate_xbl(client, &ms_token.access_token).await?;
    let xsts_token = authorize_xsts(client, &xbl_token).await?;
    let mc_token = login_minecraft(client, &user_hash, &xsts_token).await?;
    let profile = fetch_minecraft_profile(client, &mc_token.access_token).await?;

    Ok(MsSession {
        account: Account {
            username: profile.name,
            uuid: profile.id,
        },
        minecraft_token: mc_token.access_token,
        minecraft_token_expires_at: now_unix() + mc_token.expires_in,
        ms_refresh_token: ms_token.refresh_token.clone().unwrap_or_default(),
    })
}

async fn authenticate_xbl(
    client: &reqwest::Client,
    ms_access_token: &str,
) -> Result<(String, String), String> {
    let body = json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={ms_access_token}"),
        },
        "RelyingParty": XBL_RP,
        "TokenType": "JWT",
    });

    let resp = client
        .post(XBL_AUTH_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau Xbox Live : {e}"))?;

    let status = resp.status().as_u16();
    let body_text = resp.text().await.unwrap_or_default();
    if status != 200 {
        return Err(format!(
            "L'authentification Xbox Live a échoué (HTTP {status}). {body_text}"
        ));
    }

    let parsed: XblResponse = serde_json::from_str(&body_text)
        .map_err(|e| format!("Réponse Xbox Live invalide : {e}"))?;
    let uhs = parsed
        .display_claims
        .xui
        .first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| "Réponse Xbox Live sans hash utilisateur.".to_string())?;

    Ok((parsed.token, uhs))
}

async fn authorize_xsts(client: &reqwest::Client, xbl_token: &str) -> Result<String, String> {
    let body = json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token],
        },
        "RelyingParty": XSTS_RP,
        "TokenType": "JWT",
    });

    let resp = client
        .post(XSTS_AUTH_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau XSTS : {e}"))?;

    let status = resp.status().as_u16();
    let body_text = resp.text().await.unwrap_or_default();

    if status != 200 {
        // XSTS renvoie une erreur JSON { XErr: "2148916233" } en cas de compte sans Xbox.
        let xerr = serde_json::from_str::<serde_json::Value>(&body_text)
            .ok()
            .and_then(|v| v.get("XErr").and_then(|x| x.as_str()).map(str::to_string));
        if let Some(code) = xerr {
            if let Some(msg) = xsts_error_message(&code) {
                return Err(msg);
            }
        }
        return Err(format!("L'autorisation XSTS a échoué (HTTP {status}). {body_text}"));
    }

    let parsed: XblResponse = serde_json::from_str(&body_text)
        .map_err(|e| format!("Réponse XSTS invalide : {e}"))?;
    Ok(parsed.token)
}

async fn login_minecraft(
    client: &reqwest::Client,
    user_hash: &str,
    xsts_token: &str,
) -> Result<MinecraftTokenResponse, String> {
    let body = json!({
        "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}"),
    });

    let resp = client
        .post(MINECRAFT_LOGIN_URL)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau Minecraft Services : {e}"))?;

    let status = resp.status().as_u16();
    let body_text = resp.text().await.unwrap_or_default();
    if status != 200 {
        return Err(format!(
            "L'obtention du jeton Minecraft a échoué (HTTP {status}). {body_text}"
        ));
    }

    serde_json::from_str(&body_text)
        .map_err(|e| format!("Réponse Minecraft invalide : {e}"))
}

async fn fetch_minecraft_profile(
    client: &reqwest::Client,
    mc_token: &str,
) -> Result<MinecraftProfile, String> {
    let resp = auth_bearer(client, PROFILE_URL, mc_token)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau lors de la récupération du profil : {e}"))?;

    let status = resp.status().as_u16();
    let body_text = resp.text().await.unwrap_or_default();

    if status == 404 {
        return Err(
            "Ce compte ne possède pas Minecraft Java, ou son profil est introuvable.".to_string(),
        );
    }
    if status != 200 {
        return Err(format!(
            "La récupération du profil Minecraft a échoué (HTTP {status}). {body_text}"
        ));
    }

    serde_json::from_str(&body_text).map_err(|e| format!("Réponse profil invalide : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_poll_pending_and_errors() {
        assert_eq!(
            classify_poll_response(400, r#"{"error":"authorization_pending"}"#),
            PollOutcome::Pending
        );
        assert_eq!(
            classify_poll_response(400, r#"{"error":"authorization_declined"}"#),
            PollOutcome::Declined
        );
        assert_eq!(
            classify_poll_response(400, r#"{"error":"expired_token"}"#),
            PollOutcome::Expired
        );
        assert_eq!(
            classify_poll_response(400, r#"{"error":"slow_down"}"#),
            PollOutcome::SlowDown
        );
        assert!(matches!(
            classify_poll_response(400, r#"{"error":"bad_verification_code"}"#),
            PollOutcome::Failed(_)
        ));
    }

    #[test]
    fn classify_poll_success_parses_token() {
        match classify_poll_response(
            200,
            r#"{"access_token":"at","refresh_token":"rt","expires_in":3600}"#,
        ) {
            PollOutcome::Authorized(t) => {
                assert_eq!(t.access_token, "at");
                assert_eq!(t.refresh_token.as_deref(), Some("rt"));
            }
            _ => panic!("expected Authorized"),
        }
    }

    #[test]
    fn xsts_error_codes_map_to_french_messages() {
        assert!(xsts_error_message("2148916233").is_some());
        assert!(xsts_error_message("2148916238").is_some());
        assert!(xsts_error_message("9999999999").is_none());
    }

    #[test]
    fn default_client_id_is_empty_without_env() {
        // S'assure que la config par défaut force le message d'erreur explicite.
        std::env::remove_var("TERMINATOR_MS_CLIENT_ID");
        assert_eq!(msa_client_id(), "");
    }
}
