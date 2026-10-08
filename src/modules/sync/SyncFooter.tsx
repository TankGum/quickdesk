import { useEffect, useState } from "react";

import { relativeTime } from "../../shared/time";
import { useSyncStatus } from "./useSyncStatus";

/** One-line sync status for the bottom of the main window. */
export function SyncFooter({ onOpenSettings }: { onOpenSettings: () => void }) {
  const status = useSyncStatus();
  const [, tick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => tick((n) => n + 1), 30_000);
    return () => clearInterval(id);
  }, []);
  if (!status) return null;

  const text = (() => {
    switch (status.state) {
      case "disabled":
        return "Sync off";
      case "locked":
        return "Sync locked: enter your passphrase";
      case "syncing":
        return "Syncing…";
      case "offline":
        return `Offline: will retry (${status.lastError ?? ""})`;
      case "error":
        return `Sync error: ${status.lastError ?? "unknown"}`;
      default:
        return status.lastSyncAt ? `Synced · ${relativeTime(status.lastSyncAt)}` : "Sync ready";
    }
  })();
  return (
    <footer className={`sync-footer ${status.state}`} onClick={onOpenSettings} title="Sync settings">
      <span className="dot" /> {text}
    </footer>
  );
}
