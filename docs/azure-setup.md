# Azure AD — configuration du login Microsoft (device flow)

Le login Microsoft de TerminatorV1 utilise l'**OAuth 2.0 device code flow**.
Il requiert une application enregistrée dans Azure AD (Microsoft Entra ID) dont tu
possèdes le `client_id`. Sans app enregistrée, Microsoft refuse la requête et
l'utilisation du `client_id` d'un tiers est contraire aux conditions d'utilisation.

---

## 1. Créer l'application

1. Va sur https://portal.azure.com et connecte-toi avec ton compte Microsoft
   (celui qui possède le Minecraft à utiliser).
2. Barre de recherche : **Microsoft Entra ID** (ex Azure Active Directory).
3. Menu latéral : **App registrations** → **New registration**.
4. Remplis :
   - **Name** : `TerminatorV1`
   - **Supported account types** : *Personal Microsoft accounts only*
     (comptes consommateurs — requis pour Xbox/Minecraft).
   - **Redirect URI** : laisser vide (le device flow n'en a pas besoin).
5. Clique **Register**. La page de l'app s'affiche.

---

## 2. Récupérer le client_id

Sur la page de l'app, copie la valeur **Application (client) ID**
(format GUID, ex. `a1b2c3d4-...`). C'est le `client_id` à configurer dans le
launcher (étape 4).

---

## 3. Activer le device flow (optionnel, souvent actif par défaut)

1. Dans l'app Azure : **Manage** → **Authentication**.
2. Section **Advanced settings** → active **Allow public client flows**.
   (Pour un client public desktop, le device flow est activé par ce réglage.)
3. **Save**.

> Note : le device flow avec l'endpoint `consumers` fonctionne sur les comptes
> personnels sans permission Xbox pré-déclarée. La permission `XboxLive.signin`
> est demandée dynamiquement au consentement et fonctionne sans ajout manuel.
> Si tu rencontres `invalid_grant`/`unauthorized_client`, active bien le point 3.

---

## 4. Configurer le client_id dans TerminatorV1

Le launcher lit le client_id via la variable d'environnement suivante
(fallback sur une constante compilée) :

```bash
export TERMINATOR_MS_CLIENT_ID="<ton-application-client-id>"
```

Puis lancer `npm run tauri dev` depuis le même terminal.

### Où le code le lit
`launcher/src-tauri/src/commands/msauth.rs` → `msa_client_id()` :
`TERMINATOR_MS_CLIENT_ID` si définie, sinon `MSA_CLIENT_ID` (constante compilée).
Pour un usage durable, mets ta vraie valeur dans `MSA_CLIENT_ID` — mais ne commit
jamais un secret ; un client_id public n'est pas un secret, mais reste propre à ton
app. Préfère l'env var.

---

## 5. Sécurité & bonne pratique

- Le `client_id` d'une app **public client** n'est **pas** un secret (il est
  exposé dans le binaire de toute façon). Ne le traite pas comme un mot de passe.
- Ne stocke **jamais** le `refresh_token` en clair dans le repo. Il sera conservé
  côté client (Phase 3) dans le dossier de config du launcher, jamais committé.
- Scopes demandés : `XboxLive.signin offline_access` (le 2e donne le refresh_token).

---

## 6. Vérifier

1. `TERMINATOR_MS_CLIENT_ID=... npm run tauri dev`
2. Paramètres → Compte → **Se connecter avec Microsoft**.
3. Un code + URL s'affichent → ouvre l'URL dans le navigateur, entre le code.
4. Retourne au launcher : le compte doit s'afficher (pseudo + UUID).

Si erreur :
- `invalid_client` → client_id incorrect.
- `unauthorized_client`/`invalid_grant` → public client flow désactivé (étape 3).
- `user_not_found` (XSTS, 401 XErr 2148916233) → le compte n'a pas de profil Xbox.
- 404 au profil Minecraft → le compte ne possède pas Minecraft Java.
