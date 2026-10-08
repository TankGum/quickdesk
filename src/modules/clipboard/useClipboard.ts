import { useCallback, useEffect, useState } from "react";

import { api, ClipEntry, ClipKind, ClipStatus, errorMessage, useBackendEvent } from "../../shared/ipc";

/** Clipboard entries matching `query` (and `kind`), kept fresh via backend events. */
export function useClipboard(query: string, limit: number, kind?: ClipKind) {
  const [entries, setEntries] = useState<ClipEntry[]>([]);
  const [status, setStatus] = useState<ClipStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    const q = query.trim();
    (q ? api.clipSearch(q, limit, kind) : api.clipList(limit, kind))
      .then((e) => {
        setEntries(e);
        setError(null);
      })
      .catch((e) => setError(errorMessage(e)));
    void api.clipStatus().then(setStatus);
  }, [query, limit, kind]);

  useEffect(() => {
    const id = setTimeout(refresh, 60);
    return () => clearTimeout(id);
  }, [refresh]);
  useBackendEvent("clipboard://changed", refresh);

  return { entries, status, error, setError, refresh };
}

/** One-line, whitespace-collapsed preview. */
export function oneLine(text: string, max = 160): string {
  const flat = text.replace(/\s+/g, " ").trim();
  return flat.length > max ? flat.slice(0, max) + "…" : flat;
}
