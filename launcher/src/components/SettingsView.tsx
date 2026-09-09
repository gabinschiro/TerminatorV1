import { motion } from "framer-motion";
import { useLauncherStore } from "@/store/launcher";
import { GlassPanel } from "@/components/ui/GlassPanel";
import { openUrl } from "@tauri-apps/plugin-opener";

const RAM_OPTIONS = [2048, 3072, 4096, 6144, 8192, 12288] as const;
const VERSION_OPTIONS = ["1.21.4", "1.21.1", "1.20.6"] as const;

export function SettingsView() {
  const { ramMb, setRam, version, setVersion, account, logout } =
    useLauncherStore();

  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, ease: "easeOut" }}
      className="mx-auto flex max-w-2xl flex-col gap-6"
    >
      <h1 className="text-3xl font-bold tracking-tight">Paramètres</h1>

      <GlassPanel className="p-6">
        <h2 className="mb-4 text-sm font-semibold uppercase tracking-wide text-text-muted">
          Mémoire allouée
        </h2>
        <div className="flex flex-wrap gap-2">
          {RAM_OPTIONS.map((mb) => (
            <button
              key={mb}
              onClick={() => setRam(mb)}
              className={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${
                ramMb === mb
                  ? "bg-accent text-white"
                  : "bg-surface-2 text-text-muted hover:bg-white/10"
              }`}
            >
              {(mb / 1024).toFixed(0)} Go
            </button>
          ))}
        </div>
      </GlassPanel>

      <GlassPanel className="p-6">
        <h2 className="mb-4 text-sm font-semibold uppercase tracking-wide text-text-muted">
          Version du jeu
        </h2>
        <div className="flex flex-wrap gap-2">
          {VERSION_OPTIONS.map((v) => (
            <button
              key={v}
              onClick={() => setVersion(v)}
              className={`rounded-xl px-4 py-2 text-sm font-medium transition-colors ${
                version === v
                  ? "bg-accent text-white"
                  : "bg-surface-2 text-text-muted hover:bg-white/10"
              }`}
            >
              {v}
            </button>
          ))}
        </div>
      </GlassPanel>

      <GlassPanel className="p-6">
        <h2 className="mb-4 text-sm font-semibold uppercase tracking-wide text-text-muted">
          Compte
        </h2>
        {account ? <AccountPanel account={account} onLogout={logout} /> : <LoginPanel />}
      </GlassPanel>
    </motion.div>
  );
}

function AccountPanel({
  account,
  onLogout,
}: {
  account: { username: string; uuid: string };
  onLogout: () => Promise<void>;
}) {
  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="font-medium">{account.username}</p>
        <p className="text-xs text-text-muted">{account.uuid}</p>
      </div>
      <button
        onClick={onLogout}
        className="rounded-xl bg-surface-2 px-4 py-2 text-sm font-medium hover:bg-white/10"
      >
        Se déconnecter
      </button>
    </div>
  );
}

function LoginPanel() {
  const { msStatus, msDevice, msError, beginMsLogin, completeMsLogin, cancelMsLogin } =
    useLauncherStore();
  const polling = msStatus === "polling";
  const showingCode = msDevice !== null;

  if (!showingCode) {
    return (
      <div className="flex flex-col gap-3">
        <p className="text-sm text-text-muted">
          Connectez votre compte Microsoft pour jouer en ligne.{" "}
          {msStatus === "error" && msError && (
            <span className="mt-1 block text-red-400">{msError}</span>
          )}
        </p>
        <button
          onClick={beginMsLogin}
          className="w-fit rounded-xl bg-accent px-5 py-2 text-sm font-semibold text-white"
        >
          Se connecter avec Microsoft
        </button>
      </div>
    );
  }

  if (!msDevice) return null;

  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-text-muted">
        Ouvrez le lien ci-dessous puis entrez ce code pour valider la connexion.
      </p>

      <div className="rounded-2xl border border-border bg-bg px-6 py-5 text-center">
        <p className="text-xs uppercase tracking-wide text-text-muted">
          Votre code
        </p>
        <p className="mt-1 font-mono text-4xl font-bold tracking-[0.3em] text-accent-soft">
          {msDevice.user_code}
        </p>
      </div>

      {msError && msStatus === "error" && (
        <p className="text-sm text-red-400">{msError}</p>
      )}

      <div className="flex gap-2">
        <button
          onClick={() => openUrl(msDevice.verification_uri)}
          className="rounded-xl bg-accent px-5 py-2 text-sm font-semibold text-white"
        >
          Ouvrir le navigateur
        </button>
        <button
          onClick={completeMsLogin}
          disabled={polling}
          className="rounded-xl bg-surface-2 px-4 py-2 text-sm font-medium hover:bg-white/10 disabled:opacity-60"
        >
          {polling ? "En attente de validation..." : "J'ai validé, continuer"}
        </button>
        <button
          onClick={cancelMsLogin}
          disabled={polling}
          className="ml-auto text-sm text-text-muted hover:underline disabled:opacity-60"
        >
          Annuler
        </button>
      </div>
    </div>
  );
}