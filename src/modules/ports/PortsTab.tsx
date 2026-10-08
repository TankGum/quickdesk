import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { api, currentWindow, errorMessage, PortEntry } from "../../shared/ipc";

const REFRESH_MS = 2000;
const TERM_GRACE_MS = 3000;

/** What a row is waiting on: confirmation, a running kill, or a force-kill offer. */
type RowState =
  | { kind: "confirm" }
  | { kind: "working"; text: string }
  | { kind: "stubborn" };

const rowKey = (e: PortEntry) => `${e.port}:${e.pid ?? e.user ?? ""}`;

export function matches(e: PortEntry, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  if (/^\d+$/.test(q)) return String(e.port).startsWith(q) || String(e.pid ?? "").startsWith(q);
  return [e.process, e.cmdline, e.user, e.container?.name, e.container?.image]
    .some((f) => f?.toLowerCase().includes(q));
}

export function PortsTab({ focusSignal }: { focusSignal: number }) {
  const [entries, setEntries] = useState<PortEntry[] | null>(null);
  const [query, setQuery] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [rows, setRows] = useState<Record<string, RowState>>({});
  const search = useRef<HTMLInputElement>(null);

  const refresh = useCallback(async () => {
    try {
      setEntries(await api.portsScan());
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = setInterval(() => {
      // Only scan while the main window is actually on screen.
      void currentWindow.isVisible().then((v) => {
        if (v) void refresh();
      });
    }, REFRESH_MS);
    return () => clearInterval(id);
  }, [refresh]);

  useEffect(() => {
    search.current?.focus();
    void refresh();
  }, [focusSignal, refresh]);

  useEffect(() => {
    if (!notice) return;
    const id = setTimeout(() => setNotice(null), 2500);
    return () => clearTimeout(id);
  }, [notice]);

  const setRow = (key: string, state: RowState | null) =>
    setRows((r) => {
      const next = { ...r };
      if (state) next[key] = state;
      else delete next[key];
      return next;
    });

  const copy = (text: string, what: string) =>
    api.clipboardWrite(text).then(
      () => setNotice(`Copied ${what}`),
      (e) => setError(errorMessage(e)),
    );

  const kill = async (e: PortEntry, force: boolean) => {
    const key = rowKey(e);
    const pid = e.pid!;
    setRow(key, { kind: "working", text: force ? "Force killing…" : "Stopping…" });
    try {
      await api.portsKill(pid, force);
      const deadline = Date.now() + (force ? 1000 : TERM_GRACE_MS);
      while (Date.now() < deadline) {
        if (!(await api.portsIsAlive(pid))) {
          setRow(key, null);
          setNotice(`${e.process ?? "Process"} (PID ${pid}) stopped`);
          return void refresh();
        }
        await new Promise((r) => setTimeout(r, 250));
      }
      setRow(key, { kind: "stubborn" });
    } catch (err) {
      setRow(key, null);
      setError(errorMessage(err));
    }
  };

  const stopContainer = async (e: PortEntry) => {
    const key = rowKey(e);
    setRow(key, { kind: "working", text: "Stopping container…" });
    try {
      await api.portsStopContainer(e.container!.id);
      setNotice(`Container ${e.container!.name} stopped`);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setRow(key, null);
      void refresh();
    }
  };

  const visible = useMemo(() => (entries ?? []).filter((e) => matches(e, query)), [entries, query]);

  return (
    <div className="ports">
      <div className="toolbar">
        <input
          ref={search}
          className="search"
          placeholder="Filter by port, PID, process, container…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Escape" && setQuery("")}
        />
        <button className="btn" onClick={() => void refresh()} title="Rescan now">
          ↻
        </button>
      </div>
      {error && <div className="banner error">{error}</div>}
      {entries === null ? (
        <div className="empty">Scanning…</div>
      ) : visible.length === 0 ? (
        <div className="empty">{query ? `Nothing is listening on “${query}”.` : "No listening TCP ports."}</div>
      ) : (
        <table className="port-table">
          <thead>
            <tr>
              <th>Port</th>
              <th>Owner</th>
              <th>PID</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {visible.map((e) => {
              const key = rowKey(e);
              const state = rows[key];
              return (
                <tr key={key}>
                  <td className="port">
                    <b>{e.port}</b>
                    <div className="addrs">{e.addrs.join(" · ")}</div>
                  </td>
                  <td className="owner">
                    <Owner e={e} />
                  </td>
                  <td className="pid">{e.pid ?? "—"}</td>
                  <td className="actions">
                    {state?.kind === "confirm" ? (
                      <>
                        <span className="muted">{e.container ? "Stop container?" : `Kill ${e.process ?? "process"}?`}</span>
                        <button
                          className="btn danger"
                          onClick={() => void (e.container ? stopContainer(e) : kill(e, false))}
                        >
                          {e.container ? "Stop" : "Kill"}
                        </button>
                        <button className="btn" onClick={() => setRow(key, null)}>
                          Cancel
                        </button>
                      </>
                    ) : state?.kind === "working" ? (
                      <span className="muted">{state.text}</span>
                    ) : state?.kind === "stubborn" ? (
                      <>
                        <span className="muted">Still running</span>
                        <button className="btn danger" onClick={() => void kill(e, true)}>
                          Force kill
                        </button>
                        <button className="btn" onClick={() => setRow(key, null)}>
                          Leave it
                        </button>
                      </>
                    ) : (
                      <>
                        <button className="btn" onClick={() => void api.portsOpen(e.port).catch((x) => setError(errorMessage(x)))}>
                          Open
                        </button>
                        {e.pid !== null && (
                          <button className="btn" onClick={() => void copy(String(e.pid), "PID")}>
                            Copy PID
                          </button>
                        )}
                        {e.cmdline && (
                          <button className="btn" onClick={() => void copy(e.cmdline!, "command")}>
                            Copy cmd
                          </button>
                        )}
                        {(e.pid !== null || e.container) && (
                          <button className="btn danger" onClick={() => setRow(key, { kind: "confirm" })}>
                            {e.container ? "Stop" : "Kill"}
                          </button>
                        )}
                      </>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
      {notice && <div className="toast">{notice}</div>}
    </div>
  );
}

function Owner({ e }: { e: PortEntry }) {
  if (e.container) {
    return (
      <>
        <div>
          🐳 <b>{e.container.name}</b> <span className="muted">→ :{e.container.privatePort}</span>
        </div>
        <div className="cmd" title={e.container.image}>
          {e.container.image}
        </div>
      </>
    );
  }
  if (e.pid === null) {
    return (
      <>
        <div>{e.user ?? "unknown"}</div>
        <div className="cmd">Process hidden: owned by another user (needs root to inspect)</div>
      </>
    );
  }
  return (
    <>
      <div>
        <b>{e.process ?? "?"}</b> {e.user && <span className="muted">· {e.user}</span>}
      </div>
      {e.cmdline && (
        <div className="cmd" title={e.cmdline}>
          {e.cmdline}
        </div>
      )}
    </>
  );
}
