import { useEffect } from "react";
import { motion } from "framer-motion";
import { useLauncherStore } from "@/store/launcher";
import { GlassPanel } from "@/components/ui/GlassPanel";

export function HomeView() {
  const {
    system,
    refreshSystem,
    account,
    version,
    ramMb,
    active,
    launching,
    progress,
    launchError,
    installAndPlay,
  } = useLauncherStore();

  useEffect(() => {
    refreshSystem();
  }, [refreshSystem]);

  const busy = active || launching;

  return (
    <div className="relative flex h-full flex-col justify-center">
      <motion.div
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, ease: "easeOut" }}
      >
        <h1 className="text-5xl font-bold leading-tight tracking-tight">
          Prêt à jouer.
        </h1>
        <p className="mt-2 max-w-md text-text-muted">
          Lance TerminatorV1 sur la version {version} avec un rendu optimisé
          et des modules avancés.
        </p>

        <div className="mt-8 flex items-center gap-4">
          <motion.button
            whileHover={{ scale: busy ? 1 : 1.03 }}
            whileTap={{ scale: busy ? 1 : 0.97 }}
            onClick={installAndPlay}
            disabled={busy}
            className="rounded-2xl bg-accent px-8 py-3 text-base font-semibold text-white shadow-lg shadow-accent/40 disabled:opacity-60"
          >
            {active
              ? "Installation..."
              : launching
                ? "Lancement..."
                : account
                  ? "Jouer"
                  : "Installer"}
          </motion.button>

          <div className="flex flex-col gap-1">
            <span className="text-xs uppercase tracking-wide text-text-muted">
              Mémoire allouée
            </span>
            <span className="text-sm font-medium">{ramMb} Mo</span>
          </div>
        </div>

        {launchError && (
          <p className="mt-4 max-w-md text-sm text-red-400">{launchError}</p>
        )}

        {active && progress && <DownloadBar progress={progress} />}
      </motion.div>

      {system && (
        <GlassPanel className="mt-12 grid max-w-lg grid-cols-3 gap-4 p-5">
          <Stat label="CPU" value={system.cpu_name.split(" ").slice(0, 2).join(" ")} />
          <Stat label="Cœurs" value={`${system.cpu_cores}`} />
          <Stat label="RAM dispo" value={`${system.total_memory_gb} Go`} />
        </GlassPanel>
      )}
    </div>
  );
}

function DownloadBar({ progress }: { progress: { percent: number } }) {
  return (
    <div className="mt-8 max-w-md">
      <div className="flex justify-between text-xs text-text-muted">
        <span>Téléchargement en cours...</span>
        <span>{progress.percent}%</span>
      </div>
      <div className="mt-2 h-2 w-full overflow-hidden rounded-full bg-surface-2">
        <motion.div
          className="h-full bg-accent"
          animate={{ width: `${progress.percent}%` }}
          transition={{ ease: "easeOut" }}
        />
      </div>
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