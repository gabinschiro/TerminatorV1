use serde::{Deserialize, Serialize};
use serde_json::Value;

const SESSION_SERVER_URL: &str = "https://sessionserver.mojang.com/session/minecraft/profile";

#[derive(Deserialize, Serialize, Clone)]
pub struct PlayerSkin {
    pub url: String,
}

#[derive(Deserialize)]
struct ProfileResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    name: String,
    properties: Vec<ProfileProperty>,
}

#[derive(Deserialize)]
struct ProfileProperty {
    name: String,
    value: String,
}

fn decode_base64_skin(encoded: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Impossible de décoder les textures : {e}"))?;
    let json: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Textures invalides : {e}"))?;
    json["textures"]["SKIN"]["url"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "Le compte n'a pas de skin défini.".to_string())
}

#[tauri::command]
pub async fn get_player_skin(uuid: String) -> Result<PlayerSkin, String> {
    let url = format!("{SESSION_SERVER_URL}/{uuid}");
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("Erreur réseau lors de la récupération du skin : {e}"))?;

    let status = resp.status().as_u16();
    if status == 404 {
        return Err("Profil introuvable sur le session server Mojang.".to_string());
    }
    let body = resp
        .text()
        .await
        .map_err(|e| format!("Lecture de la réponse impossible : {e}"))?;
    if status != 200 {
        return Err(format!(
            "La récupération du skin a échoué (HTTP {status}). {body}"
        ));
    }

    let profile: ProfileResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Réponse profil invalide : {e}"))?;

    let textures = profile
        .properties
        .iter()
        .find(|p| p.name == "textures")
        .ok_or_else(|| "Profil sans propriété textures.".to_string())?;

    Ok(PlayerSkin {
        url: decode_base64_skin(&textures.value)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Propriété "textures" encodée en base64 d'un compte réel : {textures:{SKIN:{url:"..."}}}.
    const ENCODED: &str = "eyJ0aW1lc3RhbXAiOiIwMDAwIiwicHJvZmlsZUlkIjoiMDAwIiwicHJvZmlsZU5hbWUiOiJ6cW9kZXYiLCJ0ZXh0dXJlcyI6eyJTS0lOIjp7InVybCI6Imh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvMTIzNDU2In19fQ==";

    #[test]
    fn decodes_base64_textures_to_skin_url() {
        let url = decode_base64_skin(ENCODED).expect("should decode");
        assert!(url.contains("textures.minecraft.net"));
    }

    #[test]
    fn rejects_malformed_base64() {
        assert!(decode_base64_skin("not-base64!").is_err());
    }

    #[test]
    fn profile_parses_properties() {
        let json = r#"{
            "id": "f0dac655edd143a3ac51c07980a94244",
            "name": "zqodev",
            "properties": [
                { "name": "textures", "value": "eyJ0ZXh0dXJlcyI6e319" }
            ]
        }"#;
        let profile: ProfileResponse = serde_json::from_str(json).expect("should parse");
        assert_eq!(profile.name, "zqodev");
        assert_eq!(profile.properties.len(), 1);
    }
}