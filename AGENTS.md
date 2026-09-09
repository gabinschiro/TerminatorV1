# AGENTS.md — TerminatorV1

Guide de travail pour agents IA (et humains) sur ce repo. **Production-grade.**

---

## 1. Vue d'ensemble du projet

Client Minecraft moderne + launcher, équivalent "Lunar/Badlion en mieux", from-scratch.

| Composant | Techno |
|-----------|--------|
| `launcher/` | Tauri v2 (Rust) + React + TypeScript + Tailwind + Vite |
| `client/` | Fabric + Kotlin + Mixin (Gradle + Loom) |
| `server/` | Backend Go/Node (phase 2) |

Objectif : code maintenable, testé, linté, docomenté.

---

## 2. Règles d'or (toujours respecter)

1. **Ne jamais committer** : secrets, tokens, `.env`, caches, build, config locale.
2. **Préférer éditer des fichiers existants** ; ne créer un fichier que si nécessaire.
3. **Aucun commentaire superflu** : commenter le *pourquoi*, jamais le *quoi*.
4. **Respecter les conventions** existantes avant d'innover.
5. **Toujours vérifier** avant de déclarer une tâche finie : lint + test + build.
6. **Emojis interdits** sauf demande explicite.

---

## 3. Commandes à connaître

### Client (`client/`)
```bash
gradle build          # compile + assemble
gradle runClient      # lance le client dev
gradle test           # tests unitaires (JUnit + TestKit)
gradle ktlintCheck    # lint Kotlin
gradle detekt         # analyse statique Kotlin
gradle remapJar       # produire le jar remapé (production)
```

### Launcher (`launcher/`)
```bash
npm run dev           # dev serveur frontend
npm run build         # build frontend (Vite)
npm run tauri dev     # dev app desktop
npm run tauri build   # build release
cargo clippy          # lint Rust
cargo test            # tests Rust
npm run lint          # ESLint
npm run test          # Vitest
```

### À la racine
```bash
git status && git diff   # inspecter avant toute modification
```

---

## 4. Structure du monorepo

```
launcher/  → Tauri (frontend React/TS + backend Rust)
client/    → Fabric/Kotlin client
server/    → backend (phase 2)
docs/      → architecture, guides
```

### Règles par dossier
- **`client/src/main/kotlin/net/terminator/`**
  - `core/` = moteur de modules (Module, ModuleManager, EventBus, Config). **Ne jamais casser l'API core**.
  - `modules/<categorie>/` = un dossier par module. Un module = self-contained.
  - `mixins/` = injections isolées, pas de logique métier.
- **`launcher/src-tauri/src/commands/`** = une commande Tauri par fichier.
- **`launcher/src/`** = React ; composants UI dans `components/ui/` (shadcn).

---

## 5. Conventions de code

### Kotlin / client
- Formatage via **ktlint**. Indentation 4 espaces.
- Noms : `camelCase` méthodes/var, `PascalCase` classes, `SCREAMING_SNAKE` constantes.
- Utiliser Kotlin idiomatique (data class, null-safety, sealed class pour catégories de modules).
- Chaque `Module` DOIT exposer : `name`, `category`, `keyBind`, `settings`, `onEnable/onDisable`.
- Config en JSON, chargée au startup, sauvée au shutdown + périodiquement.
- Pas de dépendances inutiles : EventBus maison, pas de framework externe.

### Rust / launcher
- Formatage via **rustfmt**, lint via **clippy** (`cargo clippy -- -D warnings`).
- Utiliser des types forts (`struct`/`enum`) pour SysInfo, Auth, Progress.
- Toute commande Tauri = fonction publique dans `commands/*` + bindings TS générés.
- Gestion d'erreur : `Result<_, AppError>` avec messages clairs côté front.

### TypeScript / React
- Strict TypeScript, `noUncheckedIndexedAccess`.
- État global via **Zustand**, pas de prop-drilling.
- Styles : **Tailwind** avec design tokens dans `@theme` (pas de couleur en dur).
- Composants réutilisables dans `components/ui/` (shadcn).

### Convention de nommage repos/fichiers
- Kebab-case pour dossiers (`pvp/`, `visuals/`), PascalCase pour composants React.
- Fichiers Rust : snake_case. Kotlin : nom de classe/objet.

---

## 6. Commits & Git

- **Commits conventionnels** : `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`, `test:`.
- Ne commit que **si demandé explicitement** par l'utilisateur.
- Inspecter `git status` + `git diff` avant de committer. Ne jamais committer l'accidentel.
- Stager uniquement les fichiers voulus.
- Message de commit concis, aligné sur le style du repo.

---

## 7. Workflow de modification (à suivre à chaque tâche)

1. **Comprendre** : lire les fichiers concernés + contexte avant de toucher.
2. **Vérifier les conventions** de la zone (imports, style, libs existantes).
3. **Implémenter** en mimant le style existant.
4. **Vérifier** : exécuter lint + tests + build de la zone modifiée.
5. **Documenter** toute API/commande Tauri ajoutée.
6. **Ne pas** déclarer "fait" sans le point 4.

---

## 8. Qualité & production

- **CI** : lint + test + build sur GitHub Actions avant merge.
- **Releases** : semantic-release, binaire signé, `tauri updater`.
- **Secrets** : jamais dans le code. Utiliser variables d'environnement/vault.
- **Tests** :
  - Client : JUnit + TestKit Fabric (mock MC pour les modules).
  - Launcher : Vitest (front) + cargo test (Rust).
  - Toute nouvelle commande Tauri Rust = test unitaire.

---

## 9. Pièges connus / notes

- Gradle déjà configuré avec **Loom** et **Gradle 9.5.1** → ne pas le casser.
- Le client part **from scratch** : monter `core/` d'abord, modules ensuite.
- Ne pas casser le contrat Launcher↔Client (args `--terminator-*` + dossier `.terminator/`).
- Anti-cheat / modules PvP : respecter les règles serveur, version légale (à valider).

---

## 10. Définition of Done (DoD)

Une tâche est terminée uniquement si :
- [ ] Code implémenté en respectant les conventions du §5
- [ ] Lint passe (ktlint/clippy/eslint)
- [ ] Tests passent
- [ ] Build OK (client : `gradle build`, launcher : `npm run build` + `cargo clippy`)
- [ ] Toute API/commande nouvelle documentée
- [ ] Pas de secrets, pas de fichiers générés committés