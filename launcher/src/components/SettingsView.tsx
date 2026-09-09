import { motion } from "framer-motion";
import { useLauncherStore } from "@/store/launcher";
import { GlassPanel } from "@/components/ui/GlassPanel";

const RAM_OPTIONS = [2048, 3072, 4096, 6144, 8192, 12288] as const;
const VERSION_OPTIONS = ["1.21.4", "1.21.1", "1.20.6"] as const;

export function SettingsView() {
  const { ramMb, setRam, version, setVersion, account, login, logout } =
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
        {account ? (
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium">{account.username}</p>
              <p className="text-xs text-text-muted">{account.uuid}</p>
            </div>
            <button
              onClick={logout}
              className="rounded-xl bg-surface-2 px-4 py-2 text-sm font-medium hover:bg-white/10"
            >
              Se déconnecter
            </button>
          </div>
        ) : (
          <LoginForm onLogin={login} />
        )}
      </GlassPanel>
    </motion.div>
  );
}

function LoginForm({ onLogin }: { onLogin: (code: string) => Promise<void> }) {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-text-muted">
        Entrez votre code de vérification Microsoft pour vous connecter.
      </p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          const input = e.currentTarget.elements.namedItem(
            "deviceCode",
          ) as HTMLInputElement;
          onLogin(input.value);
        }}
        className="flex gap-2"
      >
        <input
          name="deviceCode"
          placeholder="Code de vérification"
          className="flex-1 rounded-xl border border-border bg-surface-2 px-4 py-2 text-sm outline-none focus:border-accent"
        />
        <button
          type="submit"
          className="rounded-xl bg-accent px-5 py-2 text-sm font-semibold text-white"
        >
          Connecter
        </button>
      </form>
    </div>
  );
}