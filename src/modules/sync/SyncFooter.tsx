import { useEffect, useState } from "react";

import { useI18n } from "../../shared/i18n";
import { relativeTime } from "../../shared/time";
import { useSyncStatus } from "./useSyncStatus";

/** One-line sync status for the bottom of the main window. */
export function SyncFooter({ onOpenSettings }: { onOpenSettings: () => void }) {
  const { t } = useI18n();
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
        return t("sync.footer.off");
      case "locked":
        return t("sync.footer.locked");
      case "syncing":
        return t("sync.footer.syncing");
      case "offline":
        return t("sync.footer.offline", { error: status.lastError ?? "" });
      case "error":
        return t("sync.footer.error", { error: status.lastError ?? "" });
      default:
        return status.lastSyncAt ? t("sync.footer.synced", { time: relativeTime(status.lastSyncAt) }) : t("sync.footer.ready");
    }
  })();
  return (
    <footer className={`sync-footer ${status.state}`} onClick={onOpenSettings} title={t("sync.footer.title")}>
      <span className="dot" /> {text}
    </footer>
  );
}
