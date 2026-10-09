import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { api, UpdateInfo } from "../../shared/ipc";

/** Update state, kept current by `update://status` events. */
export function useUpdate() {
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  useEffect(() => {
    void api.updateStatus().then(setInfo);
    const unlisten = listen<UpdateInfo>("update://status", (e) => setInfo(e.payload));
    return () => {
      void unlisten.then((f) => f());
    };
  }, []);
  return [info, setInfo] as const;
}

/** The version an update action applies to, if any. */
export function pendingVersion(info: UpdateInfo | null): string | null {
  const s = info?.status;
  if (!s) return null;
  if (s.state === "available" || s.state === "downloading" || s.state === "installing") return s.version;
  if (s.state === "error") return s.version;
  return null;
}
