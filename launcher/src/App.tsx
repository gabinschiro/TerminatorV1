import { useEffect } from "react";
import { motion } from "framer-motion";
import { useLauncherStore } from "@/store/launcher";
import { GlassPanel } from "@/components/ui/GlassPanel";

const NAV_ITEMS = ["Accueil", "Actualités", "Mods", "Paramètres"] as const;

export default function App() {
  const { system, refreshSystem, accountName, setAccount, version, ramMb } =
  useLauncherStore();

  useEffect(() => {
    refreshSystem();
  }, [refreshSystem]);

  return (
    <div className="flex h-full overflow-hidden bg-bg text-text">
      <nav className="flex w-60 flex-col gap-2 border-r border-border p-4">
        <div className="mb-6 flex items-center gap-2 px-2">
          <span className="grid h-9 w-9 place-items-center rounded-xl bg-accent font-bold">
            T
          </span>
          <span className="text-lg font-semibold tracking-tight">
            TerminatorV1
          </span>
        </div>

        {NAV_ITEMS.map((item) => (
          <button
            key={item}
            className="rounded-xl px-3 py-2 text-left text-sm text-text-muted transition-colors hover:bg-surface-2 hover:text-text"
          >
            {item}
          </button>
        ))}

        <div className="mt-auto rounded-xl bg-surface px-3 py-3">
          <p className="text-sm font-medium">
            {accountName ?? "Non connecté"}
          </p>
          <button
            onClick={() => setAccount("Joueur123")}
            className="mt-1 text-xs text-accent-soft hover:underline"
          >
            Se connecter
          </button>
        </div>
      </nav>

      <main className="relative flex-1 overflow-y-auto">
        <div className="pointer-events-none absolute -top-40 left-1/2 h-96 w-96 -translate-x-1/2 rounded-full bg-accent/20 blur-3xl" />
        <div className="pointer-events-none absolute right-0 top-1/3 h-72 w-72 rounded-full bg-cyan-glow/10 blur-3xl" />

        <div className="relative flex h-full flex-col justify-center px-12">
          <motion.div
            initial={{ opacity: 0, y: 16 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, ease: "easeOut" }}
          >
            <h1 className="text-5xl font-bold leading-tight tracking-tight">
              Prêt à jouer.
            </h1>
            <p className="mt-2 max-w-md text-text-muted">
              Lance TerminatorV1 sur la version {version} avec un rendu optimisé et
              des modules avancés.
            </p>

            <div className="mt-8 flex items-center gap-4">
              <motion.button
                whileHover={{ scale: 1.03 }}
                whileTap={{ scale: 0.97 }}
                className="rounded-2xl bg-accent px-8 py-3 text-base font-semibold text-white shadow-lg shadow-accent/40"
              >
                Jouer
              </motion.button>

              <div className="flex flex-col gap-1">
                <span className="text-xs uppercase tracking-wide text-text-muted">
                  Mémoire allouée
                </span>
                <span className="text-sm font-medium">
                  {ramMb} Mo
                </span>
              </div>
            </div>
          </motion.div>

          {system && (
            <GlassPanel className="mt-12 grid max-w-lg grid-cols-3 gap-4 p-5">
              <Stat label="CPU" value={system.cpu_name.split(" ").slice(0, 2).join(" ")} />
              <Stat label="Cœurs" value={`${system.cpu_cores}`} />
              <Stat label="RAM dispo" value={`${system.total_memory_gb} Go`} />
            </GlassPanel>
          )}
        </div>
      </main>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-xs uppercase tracking-wide text-text-muted">
        {label}
      </span>
      <span className="truncate text-sm font-medium">{value}</span>
    </div>
  );
}