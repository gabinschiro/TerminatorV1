import { useEffect, useState } from "react";
import { useLauncherStore } from "@/store/launcher";
import { HomeView } from "@/components/HomeView";
import { SettingsView } from "@/components/SettingsView";
import { Avatar } from "@/components/ui/Avatar";

type View = "home" | "settings";

const NAV: { id: View; label: string }[] = [
  { id: "home", label: "Accueil" },
  { id: "settings", label: "Paramètres" },
];

export default function App() {
  const [view, setView] = useState<View>("home");
  const { account, skinUrl, refreshAccount } = useLauncherStore();

  useEffect(() => {
    refreshAccount();
  }, [refreshAccount]);

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

        {NAV.map((item) => (
          <button
            key={item.id}
            onClick={() => setView(item.id)}
            className={`group relative rounded-xl px-3 py-2 text-left text-sm transition-colors ${
              view === item.id
                ? "bg-accent/15 text-text"
                : "text-text-muted hover:bg-white/5 hover:text-text"
            }`}
          >
            {view === item.id && (
              <span className="absolute left-0 top-1/2 h-5 w-1 -translate-y-1/2 rounded-full bg-accent" />
            )}
            {item.label}
          </button>
        ))}

        <div className="mt-auto flex items-center gap-3 rounded-xl bg-surface px-3 py-3">
          {skinUrl && <Avatar skinUrl={skinUrl} size={36} />}
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium">
              {account?.username ?? "Non connecté"}
            </p>
            {!account && (
              <button
                onClick={() => setView("settings")}
                className="mt-0.5 text-xs text-accent-soft hover:underline"
              >
                Se connecter
              </button>
            )}
          </div>
        </div>
      </nav>

      <main className="relative flex-1 overflow-y-auto">
        <div className="pointer-events-none absolute -top-40 left-1/2 h-96 w-96 -translate-x-1/2 rounded-full bg-[radial-gradient(circle,rgba(124,58,237,0.25),transparent_70%)]" />
        <div className="pointer-events-none absolute right-0 top-1/3 h-72 w-72 rounded-full bg-[radial-gradient(circle,rgba(34,211,238,0.12),transparent_70%)]" />

        <div className="relative flex h-full px-12 py-10">
          {view === "home" ? <HomeView /> : <SettingsView />}
        </div>
      </main>
    </div>
  );
}