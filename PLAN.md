# TerminatorV1 — Plan d'architecture complet

Un client Minecraft moderne + launcher, équivalent "Lunar/Badlion, en mieux", stack
from-scratch, production-grade.

---

## 1. Vision & différenciateurs

| Pillier | Ambition |
|---------|----------|
| **UI/UX** | Launcher ultra stylée (motion, glassmorphism, temps réel) |
| **Performance** | Client Fabric optimisé, pas de bloat |
| **Modularité** | Modules désactivables (pas de monolithe) |
| **Plateforme** | Comptes, marketplace, anti-cheat serveur |

---

## 2. Architecture monorepo (2 apps)

```
terminator-v1/
├── launcher/                 # Tauri (Rust + React + Tailwind)
├── client/                   # Client Minecraft Fabric (Kotlin)
├── server/                   # Backend (comptes, telemetry, mods CDN)  [futur]
├── docs/
└── AGENTS.md
```

### Repos séparés conseillés (via submodules ou org GitHub)
- `terminator-launcher`
- `terminator-client`
- `terminator-server`

---

## 3. Launcher — Tauri + React

### Stack
| Couche | Techno |
|--------|--------|
| Shell | **Tauri v2** (Rust) |
| Frontend | **React + TypeScript** |
| Build | **Vite** |
| Style | **Tailwind CSS v4** + Framer Motion + shadcn/ui |
| État | **Zustand** (leger) |
| Routing | React Router |

### Rust (backend launcher) — commandes Tauri
- `launch_game(profile, auth) -> Handle` — spawn du client avec args
- `download_assets(version, mirror) -> Progress` — téléchargement parallèle
- `get_system_info() -> SysInfo` — CPU/GPU/RAM/OS via crate `sysinfo`
- `get_minecraft_versions() -> Vec<Version>`
- `auth_microsoft() -> Account` — OAuth device flow
- `install_client() -> Progress` — bootstrap du client from-scratch
- `update_check() -> UpdateInfo` — via `tauri-plugin-updater`

### Structure frontend
```
launcher/src/
├── main.rs
├── lib.rs
├── commands/
│   ├── auth.rs
│   ├── download.rs
│   ├── system.rs
│   └── launch.rs
└── launcher/
    ├── src/
    │   ├── App.tsx
    │   ├── components/
    │   │   ├── Shell/        (sidebar, window controls)
    │   │   ├── Home/         (play button, hero, temps réel)
    │   │   ├── News/         (changelog, annonces)
    │   │   ├── Mods/         (installer/désinstaller modules)
    │   │   ├── Settings/     (RAM, version, compte)
    │   │   └── ui/           (shadcn primitives)
    │   ├── store/            (zustand)
    │   └── lib/              (tauri API bindings)
```

### Principe UI "ultra stylée"
- **Glassmorphism + depth** : fonds flous, overlays gradient.
- **Motion** : Framer Motion pour transitions de vues, hover, micro-interactions.
- **Temps réel** : infos système + état du download en live (event stream Tauri).
- **Dark-first** : thème sombre profond, accent violet/cyan.
- **Design tokens** : Tailwind `@theme` centralisé (pas de couleurs en dur).

---

## 4. Client — Minecraft from scratch (Fabric + Kotlin)

### Version cible
- **Minecraft 1.21.x** (dernière stable) — LTS performance.

### Stack
| Couche | Techno |
|--------|--------|
| Loader | **Fabric Loader** |
| Mappings | **Yarn** (pour dev, plus lisible) |
| Modding API | **Fabric API** |
| Langage | **Kotlin** (+ `fabric-language-kotlin`) |
| Injection | **Mixin** |
| Build | **Gradle + Loom** (déjà présent, Gradle 9.5.1) |

### Architecture des modules (découplés)
```
client/
├── build.gradle.kts
├── gradle.properties
├── settings.gradle.kts
├── src/main/
│   ├── kotlin/net/terminator/
│   │   ├── TerminatorClient.kt     # entrypoint (ModInitializer)
│   │   ├── core/                   # moteur de modules
│   │   │   ├── Module.java         # interface module
│   │   │   ├── ModuleManager.kt
│   │   │   ├── ModuleConfig.kt     # config sérialisée JSON
│   │   │   └── event/              # EventBus (custom, sans dépendances)
│   │   ├── modules/                # chaque module = dossier
│   │   │   ├── hud/                # FPS, CPS, coords, ping
│   │   │   ├── pvp/                # aim assist, reach, hitboxes, aura
│   │   │   ├── visuals/            # fullbright, clear sky, no fov
│   │   │   ├── movement/           # sprint, bhop, speed
│   │   │   ├── chat/               # clic-copy, streaming mode
│   │   │   └── settings/           # toggle HUD, keybinds, search
│   │   ├── render/                 # rendu custom HUD, fonts, shaders
│   │   ├── gui/                    # écran de config module
│   │   └── mixins/                 # mixins Injections
│   └── resources/
│       ├── terminator.mixins.json
│       ├── fabric.mod.json
│       └── assets/...
```

### Moteur de modules (le coeur)
- **Interface `Module`** : `name`, `category`, `enabled`, `keyBind`, `onEnable()/onDisable()`, `settings`.
- **`ModuleManager`** : registre central, toggle par keybind/commande, sauvegarde config.
- **`EventBus`** : bus d'événements synchrone (tick, render, packet, input) — implémentation maison, zéro dépendance, typesafe via Kotlin.
- **Config** : JSON par module + un `config.json` global, chargé au startup, sauvé périodiquement et au shutdown.

### Mixins (points d'injection clés)
- `HudRenderCallback` — HUD custom.
- `MinecraftClient` (tick / runTick) — boucle modules.
- `ClientPlayerEntity` (movement) — modules movement.
- `AbstractClientPlayerEntity` (hitbox) — pvp.
- `MixinInjectedHud`, `MixinEntityRenderDispatcher` (ESP/hitboxes).
- Des packages `compat` pour garder les mixins isolés des modules.

---

## 5. Échange Launcher ↔ Client
- Le launcher passe des **args JVM** au client : `--terminator-version`, `--terminator-config`, `--terminator-auth`.
- Le client lit sa config depuis le dossier `.terminator/` partagé.
- API REST future : compte, skins, marketplace via `server/`.

---

## 6. Backend serveur (phase 2)
| Module | Techno |
|--------|--------|
| API | **Go** (Gin) ou **Node** (Fastify) |
| DB | **PostgreSQL** + Redis (cache) |
| Auth | OAuth Microsoft + JWT |
| Anti-cheat | Plugin **Paper/Velocity** |
| CDN | S3-compatible (R2/MinIO) pour assets & mods |

---

## 7. Roadmap par phases

### Phase 1 — Fondations (client)
- [ ] Setup Gradle/Loom + Fabric + Kotlin + Mixin fonctionnels (build OK)
- [ ] `TerminatorClient` + module `core` (Module, ModuleManager, EventBus)
- [ ] Module HUD minimal (FPS, coords) + config JSON
- [ ] EventBus tick/render fonctionnel

### Phase 2 — Launcher
- [x] Tauri v2 + React + Tailwind scaffold
- [x] Commands Rust : sysinfo, download, launch, auth
- [x] UI "hero" home + sidebar + settings
- [x] Streaming download + progress live

> TODO (hors phase) : l'auth est un squelette — implémenter le vrai OAuth Microsoft device-flow (compte, UUID, token, refresh) en Phase 4/5.

### Phase 3 — Intégration
- [ ] Launcher → lance le client with args
- [ ] Install client (bootstrap) depuis CDN
- [ ] Mise à jour client (tauri updater)

### Phase 4 — Modules avancés
- [ ] PvP pack (aim, hitboxes, aura) [VERSION LEGALE : à discuter]
- [ ] Visuals pack, Movement pack
- [ ] HUD configurable + écran de config in-game

### Phase 5 — Plateforme
- [ ] Backend comptes/auth
- [ ] Marketplace mods
- [ ] Anti-cheat serveur

---

## 8. Outillage & qualité (production-grade)
| Besoin | Outil |
|--------|-------|
| Lint Kotlin | **ktlint** + detekt |
| Lint JS/TS | **ESLint** + Prettier |
| Lint Rust | **clippy** + rustfmt |
| Tests client | JUnit + TestKit de Fabric (mock MC) |
| Tests launcher | Vitest (front) + cargo test (rust) |
| CI | **GitHub Actions** (lint + test + build) |
| Releases | semantic-release + GitHub Releases |
| Config versionnée | *n'ajouter aucune config locale dans git* |

### Conventions (détaillées dans AGENTS.md)
- Convention de nommage, formatage, PR, commits conventionnels.
- Interdiction de committer des secrets, caches, build.
- Doc obligatoire sur les API/commandes Tauri.

---

## 9. État actuel du repo
- Dossier vide (hors `.gradle` caches) — à partir de zéro.
- Gradle 9.5.1 + loom-cache déjà présents → bootstrap client en premier.
```
```