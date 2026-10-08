import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { api, SyncStatus } from "../../shared/ipc";

export function useSyncStatus() {
  const [status, setStatus] = useState<SyncStatus | null>(null);
  useEffect(() => {
    void api.syncStatus().then(setStatus);
    const unlisten = listen<SyncStatus>("sync://status", (e) => setStatus(e.payload));
    return () => {
      void unlisten.then((f) => f());
    };
  }, []);
  return status;
}
