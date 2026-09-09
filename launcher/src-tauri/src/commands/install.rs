use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

const PROGRESS_EVENT: &str = "download://progress";
const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2/versions/loader";
const FABRIC_API_LATEST: &str =
    "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/maven-metadata.xml";
const ASSETS_BASE: &str = "https://resources.download.minecraft.net";
const OUR_MOD_URL: &str =
    "https://github.com/gabinschiro/TerminatorV1/releases/download/v0.1.0/terminator-client-0.1.0.jar";

const STAGE_CLIENT_JAR: f32 = 0.30;
const STAGE_LIBS: f32 = 0.25;
const STAGE_ASSETS: f32 = 0.30;
const STAGE_OTHER: f32 = 0.15;

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub version: String,
    pub downloaded_mb: u64,
    pub total_mb: u64,
    pub percent: u8,
    pub done: bool,
}

// ---- Mojang version manifest ----

#[derive(Deserialize)]
struct VersionManifest {
    versions: Vec<ManifestEntry>,
}

#[derive(Deserialize)]
struct ManifestEntry {
    id: String,
    url: String,
}

// ---- Mojang version json ----

#[derive(Deserialize)]
struct VersionJson {
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    downloads: VersionDownloads,
    libraries: Vec<Library>,
}

#[derive(Deserialize)]
struct AssetIndex {
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionDownloads {
    client: Artifact,
}

#[derive(Deserialize)]
struct Artifact {
    url: String,
}

#[derive(Deserialize)]
struct Library {
    name: String,
    downloads: Option<LibraryDownloads>,
    rules: Option<Vec<Rule>>,
}

#[derive(Deserialize)]
struct LibraryDownloads {
    artifact: Option<Artifact>,
}

#[derive(Deserialize)]
struct Rule {
    action: String,
    os: Option<OsRule>,
}

#[derive(Deserialize)]
struct OsRule {
    name: Option<String>,
    arch: Option<String>,
}

// ---- Fabric launcher meta ----

#[derive(Deserialize)]
struct FabricLoaderEntry {
    #[serde(rename = "launcherMeta")]
    launcher_meta: LauncherMeta,
}

#[derive(Deserialize)]
struct LauncherMeta {
    libraries: FabricLibraries,
}

#[derive(Deserialize)]
struct FabricLibraries {
    common: Vec<FabricLibrary>,
    client: Vec<FabricLibrary>,
}

#[derive(Deserialize)]
struct FabricLibrary {
    name: String,
    url: Option<String>,
}

// ---- Asset index ----

#[derive(Deserialize)]
struct AssetIndexData {
    objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
}

fn game_dir() -> PathBuf {
    std::env::var("TERMINATOR_GAME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".terminator")
        })
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .expect("reqwest client construction")
}

fn current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

fn current_arch() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86"
    }
}

// Une règle sans clause os/arch s'applique partout ; sinon elle filtre par OS courant.
fn rules_allow(rules: &[Rule]) -> bool {
    let mut allowed = false;
    let mut matched = false;
    for rule in rules {
        let os_ok = match &rule.os {
            None => true,
            Some(os) => {
                let name_ok = os.name.as_deref().is_none_or(|n| n == current_os());
                let arch_ok = os.arch.as_deref().is_none_or(|a| a == current_arch());
                name_ok && arch_ok
            }
        };
        if os_ok {
            matched = true;
            allowed = rule.action == "allow";
        }
    }
    if matched {
        allowed
    } else {
        true
    }
}

// Convertit "net.fabricmc:fabric-loader:0.19.5" ou
// "org.lwjgl:lwjgl-glfw:3.3.3:natives-linux" en chemin maven relatif.
fn maven_path(name: &str) -> PathBuf {
    let parts: Vec<&str> = name.split(':').collect();
    let group = parts.first().copied().unwrap_or("");
    let artifact = parts.get(1).copied().unwrap_or("");
    let version = parts.get(2).copied().unwrap_or("");
    let classifier = parts.get(3).copied();
    let group_path = group.replace('.', "/");
    let file = match classifier {
        Some(c) => format!("{artifact}-{version}-{c}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    PathBuf::from(format!("{group_path}/{artifact}/{version}/{file}"))
}

// En 1.21.4 les natives sont des entrées maven nommées "<artifact>:natives-<os>".
fn is_native_for_current_os(name: &str) -> bool {
    name.ends_with(&format!(":natives-{}", current_os()))
}

fn library_local_path(lib_name: &str) -> PathBuf {
    game_dir().join("libraries").join(maven_path(lib_name))
}

async fn fetch_json<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau ({url}) : {e}"))?;
    let status = resp.status().as_u16();
    if status != 200 {
        return Err(format!("HTTP {status} en interrogeant {url}"));
    }
    let body = resp.text().await.unwrap_or_default();
    serde_json::from_str(&body).map_err(|e| format!("JSON invalide depuis {url} : {e}"))
}

async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest: &PathBuf,
) -> Result<(), String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau ({url}) : {e}"))?;
    let status = resp.status().as_u16();
    if status != 200 {
        return Err(format!("HTTP {status} en téléchargeant {url}"));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Lecture du flux impossible : {e}"))?;

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Impossible de créer {} : {e}", parent.display()))?;
    }
    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("Impossible de créer {} : {e}", dest.display()))?;
    file.write_all(&bytes)
        .await
        .map_err(|e| format!("Écriture impossible : {e}"))?;
    Ok(())
}

// Extrait une archive natives (zip) dans dest. L'archive native est marquée par
// sa présence ; on ré-extraie si le répertoire cible ne contient aucun fichier.
async fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    let dest = dest.to_path_buf();
    let archive = archive.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&archive)
            .map_err(|e| format!("Impossible d'ouvrir {} : {e}", archive.display()))?;
        let mut zip = zip::ZipArchive::new(file)
            .map_err(|e| format!("Archive natives invalide : {e}"))?;
        std::fs::create_dir_all(&dest)
            .map_err(|e| format!("Impossible de créer {} : {e}", dest.display()))?;
        for i in 0..zip.len() {
            let mut entry = zip
                .by_index(i)
                .map_err(|e| format!("Entrée invalide dans l'archive : {e}"))?;
            let name = entry.name().to_string();
            // Ignore les répertoires et les fichiers meta META-INF des jars natives.
            if entry.is_dir() || name.starts_with("META-INF") {
                continue;
            }
            let out_path = dest.join(name);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Impossible de créer {} : {e}", parent.display()))?;
            }
            let mut out = std::fs::File::create(&out_path)
                .map_err(|e| format!("Impossible de créer {} : {e}", out_path.display()))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| format!("Extraction impossible : {e}"))?;
        }
        Ok(())
    })
    .await
    .map_err(|e| format!("Tâche d'extraction interrompue : {e}"))?
}

fn emit_progress(app: &AppHandle, version: &str, percent: f32) {
    let percent = percent.clamp(0.0, 100.0);
    let _ = app.emit(
        PROGRESS_EVENT,
        DownloadProgress {
            version: version.to_string(),
            downloaded_mb: percent as u64,
            total_mb: 100,
            percent: percent as u8,
            done: percent >= 100.0,
        },
    );
}

#[tauri::command]
pub async fn install_client(app: AppHandle, version: String) -> Result<DownloadProgress, String> {
    let client = http_client();
    let version_dir = game_dir().join("versions").join(&version);
    let mods_dir = game_dir().join("mods");
    let natives_dir = game_dir().join("natives");
    let assets_dir = game_dir().join("assets");
    let indexes_dir = assets_dir.join("indexes");
    let objects_dir = assets_dir.join("objects");

    let mut progress = 0.0f32;
    emit_progress(&app, &version, progress);

    // 1. Version manifest → URL du version json cible.
    let manifest: VersionManifest = fetch_json(&client, MANIFEST_URL).await?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version)
        .ok_or_else(|| format!("Version {version} introuvable sur Mojang."))?;

    let version_json: VersionJson = fetch_json(&client, &entry.url).await?;

    // Persiste le version json : launch_game le relira pour l'index d'assets.
    let version_json_path = version_dir.join(format!("{version}.json"));
    tokio::fs::create_dir_all(&version_dir)
        .await
        .map_err(|e| format!("Impossible de créer {} : {e}", version_dir.display()))?;
    let raw = client
        .get(&entry.url)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau ({}) : {e}", entry.url))?
        .text()
        .await
        .unwrap_or_default();
    tokio::fs::write(&version_json_path, raw)
        .await
        .map_err(|e| format!("Impossible d'écrire le version json : {e}"))?;

    // 2. Fabric loader + intermediary + libs fabric.
    let loader_url = format!("{FABRIC_META_URL}/{version}");
    let loaders: Vec<FabricLoaderEntry> = fetch_json(&client, &loader_url).await?;
    let loader = loaders
        .first()
        .ok_or_else(|| "Aucun loader Fabric pour cette version.".to_string())?;

    // 3. Client jar vanilla.
    let client_jar_path = version_dir.join(format!("{version}.jar"));
    if !client_jar_path.exists() {
        download_file(&client, &version_json.downloads.client.url, &client_jar_path).await?;
    }
    progress += STAGE_CLIENT_JAR * 100.0;
    emit_progress(&app, &version, progress);

    // 4. Libraries vanilla : natives (entrées ":natives-<os>") extraites, le reste au classpath.
    let mut libs = Vec::new();
    for lib in &version_json.libraries {
        let rules = lib.rules.as_deref().unwrap_or(&[]);
        if !rules_allow(rules) {
            continue;
        }
        let artifact = lib
            .downloads
            .as_ref()
            .and_then(|d| d.artifact.as_ref())
            .ok_or_else(|| format!("Artifact manquant pour {}", lib.name))?;
        let dest = library_local_path(&lib.name);
        if is_native_for_current_os(&lib.name) {
            // Les natives sont des jars (zip) : extraction dans natives/.
            // Un fichier corrompu issu d'un téléchargement interrompu est re-téléchargé.
            for attempt in 0..2 {
                if extract_zip(&dest, &natives_dir).await.is_ok() {
                    break;
                }
                if attempt == 1 {
                    return Err(format!("Extraction natives impossible pour {}", lib.name));
                }
                let _ = tokio::fs::remove_file(&dest).await;
                download_file(&client, &artifact.url, &dest).await?;
            }
        } else {
            if !dest.exists() {
                download_file(&client, &artifact.url, &dest).await?;
            }
            libs.push(dest);
        }
    }
    progress += STAGE_LIBS * 100.0;
    emit_progress(&app, &version, progress);

    // 5. Libs fabric (loader + intermediary + mappings). Sans url explicite,
    // on retombe sur le maven Mojang (libraries.minecraft.net) pour launchwrapper.
    const MOJANG_MAVEN: &str = "https://libraries.minecraft.net/";
    for lib in loader.launcher_meta.libraries.common.iter().chain(loader.launcher_meta.libraries.client.iter()) {
        let dest = library_local_path(&lib.name);
        if !dest.exists() {
            let base = lib.url.as_deref().unwrap_or(MOJANG_MAVEN);
            let url = format!("{base}{}", maven_path(&lib.name).display());
            download_file(&client, &url, &dest).await?;
        }
        libs.push(dest);
    }

    // 6. Fabric API (dernière version) + notre mod → mods/.
    let api_meta = client
        .get(FABRIC_API_LATEST)
        .send()
        .await
        .map_err(|e| format!("Erreur réseau fabric-api : {e}"))?;
    let api_body = api_meta.text().await.unwrap_or_default();
    let api_version = api_body
        .split("<latest>")
        .nth(1)
        .and_then(|s| s.split("</latest>").next())
        .ok_or_else(|| "Impossible de résoudre la version fabric-api.".to_string())?;

    let api_jar_path = mods_dir.join(format!("fabric-api-{api_version}.jar"));
    if !api_jar_path.exists() {
        let api_url = format!(
            "https://maven.fabricmc.net/net/fabricmc/fabric-api/fabric-api/{api_version}/fabric-api-{api_version}.jar"
        );
        download_file(&client, &api_url, &api_jar_path).await?;
    }

    let mod_jar_path = mods_dir.join("terminator-client-0.1.0.jar");
    if !mod_jar_path.exists() {
        download_file(&client, OUR_MOD_URL, &mod_jar_path).await?;
    }

    // 7. Assets : index + objets hachés.
    let index_dest = indexes_dir.join(format!("{}.json", version_json.asset_index.id));
    if !index_dest.exists() {
        download_file(&client, &version_json.asset_index.url, &index_dest).await?;
    }
    let index_data: AssetIndexData = fetch_json(&client, &version_json.asset_index.url).await?;
    let total_objects = index_data.objects.len().max(1);
    let mut done_objects = 0usize;
    for obj in index_data.objects.values() {
        let dest = objects_dir
            .join(&obj.hash[0..2])
            .join(&obj.hash);
        if !dest.exists() {
            let url = format!("{ASSETS_BASE}/{}/{}", &obj.hash[0..2], obj.hash);
            download_file(&client, &url, &dest).await?;
        }
        done_objects += 1;
        progress = 30.0 + STAGE_ASSETS * 100.0 * (done_objects as f32 / total_objects as f32);
        emit_progress(&app, &version, progress);
    }
    progress = 30.0 + STAGE_LIBS * 100.0 + STAGE_ASSETS * 100.0 + STAGE_OTHER * 100.0;
    emit_progress(&app, &version, progress);

    // 8. Extrait les natives zip (jar) dans natives/ si nécessaire.
    let _ = natives_dir;

    Ok(DownloadProgress {
        version,
        downloaded_mb: 100,
        total_mb: 100,
        percent: 100,
        done: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maven_path_parses_ga_v() {
        let p = maven_path("net.fabricmc:fabric-loader:0.19.5");
        assert_eq!(
            p,
            PathBuf::from("net/fabricmc/fabric-loader/0.19.5/fabric-loader-0.19.5.jar")
        );
    }

    #[test]
    fn rules_allow_defaults_true() {
        assert!(rules_allow(&[]));
    }

    #[test]
    fn rules_deny_blocks_linux() {
        let rules = vec![Rule {
            action: "disallow".into(),
            os: Some(OsRule {
                name: Some(current_os().into()),
                arch: None,
            }),
        }];
        assert!(!rules_allow(&rules));
    }
}