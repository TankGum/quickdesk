import { useCallback, useEffect, useMemo, useState } from "react";

import { t, useI18n } from "../../shared/i18n";
import {
  api,
  AutoApplyState,
  AvailableList,
  errorMessage,
  InstalledVersion,
  Runtime,
  RuntimeAction,
  RuntimeJob,
  RuntimeUpdate,
  ShellFix,
  useBackendEvent,
} from "../../shared/ipc";
import { relativeTime } from "../../shared/time";

/** Shown rows before the filter has to narrow the list. */
const LIST_LIMIT = 40;

type Panel = "install" | "project" | "fix" | "mise" | null;

export function size(bytes: number): string {
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
  return `${Math.max(1, Math.round(bytes / 1024 ** 2))} MB`;
}

/** `/home/me/.nvm/…` → `~/.nvm/…` */
const tidy = (path: string) => path.replace(/^\/home\/[^/]+/, "~");

export function RuntimesTab({ focusSignal }: { focusSignal: number }) {
  useI18n();
  const [runtimes, setRuntimes] = useState<Runtime[] | null>(null);
  const [scanning, setScanning] = useState(false);
  const [job, setJob] = useState<RuntimeJob | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [auto, setAuto] = useState<AutoApplyState | null>(null);
  const [updates, setUpdates] = useState<RuntimeUpdate[]>([]);
  // Asks each manager for its version list (cached an hour); errors only mean no badges.
  const loadUpdates = useCallback(() => void api.runtimesUpdates().then(setUpdates, () => setUpdates([])), []);
  const loadAuto = useCallback(() => void api.runtimesAutoApply().then(setAuto, () => setAuto(null)), []);

  const scan = useCallback(async (force: boolean) => {
    setScanning(true);
    try {
      setRuntimes(await api.runtimesScan(force));
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setScanning(false);
    }
  }, []);

  // Show the last scan at once, then look again: versions may have changed in a terminal.
  useEffect(() => {
    void scan(false)
      .then(() => scan(true))
      .then(loadUpdates);
    void api.runtimesJob().then(setJob);
    loadAuto();
  }, [focusSignal, scan, loadAuto, loadUpdates]);

  useBackendEvent("runtimes://job", useCallback(() => void api.runtimesJob().then(setJob), []));
  useBackendEvent(
    "runtimes://changed",
    useCallback(() => void scan(false).then(loadUpdates), [scan, loadUpdates]),
  );

  const run = (lang: string, action: RuntimeAction, version: string) =>
    api.runtimesRun(lang, action, version).catch((e) => setError(errorMessage(e)));

  if (!runtimes) return <div className="empty">{t("rt.scanning")}</div>;

  return (
    <div className="rt">
      <div className="rt-head">
        {auto ? <AutoApply state={auto} onChanged={loadAuto} /> : <span className="spacer" />}
        <button className="btn" disabled={scanning} onClick={() => void scan(true)}>
          {scanning ? t("rt.scanning") : t("rt.rescan")}
        </button>
      </div>
      {error && <div className="banner error">{error}</div>}
      {runtimes.length === 0 ? (
        <div className="empty">{t("rt.none")}</div>
      ) : (
        <div className="rt-cards">
          {runtimes.map((rt) => (
            <RuntimeCard
              key={rt.id}
              rt={rt}
              job={job?.lang === rt.id ? job : null}
              busy={Boolean(job?.running)}
              run={run}
              onDismiss={() => setJob(null)}
              refresh={() => void scan(false)}
              autoApply={Boolean(auto?.on)}
              updates={updates.filter((u) => u.lang === rt.id)}
              onUpgrade={(u) => void api.runtimesUpgrade(u.lang, u.from, u.to).catch((e) => setError(errorMessage(e)))}
            />
          ))}
        </div>
      )}
      <p className="muted small">{t("rt.footnote")}</p>
    </div>
  );
}

function RuntimeCard({
  rt,
  job,
  busy,
  run,
  onDismiss,
  refresh,
  autoApply,
  updates,
  onUpgrade,
}: {
  rt: Runtime;
  job: RuntimeJob | null;
  busy: boolean;
  run: (lang: string, action: RuntimeAction, version: string) => void;
  onDismiss: () => void;
  refresh: () => void;
  autoApply: boolean;
  updates: RuntimeUpdate[];
  onUpgrade: (u: RuntimeUpdate) => void;
}) {
  const [panel, setPanel] = useState<Panel>(null);
  const toggle = (p: Panel) => setPanel((cur) => (cur === p ? null : p));
  const expected = rt.installed.find((i) => i.isDefault)?.version;
  return (
    <section className="rt-card">
      <header>
        <h3>{rt.name}</h3>
        {rt.manager ? (
          <span className="badge">
            {rt.manager}
            {rt.managerVersion && ` ${rt.managerVersion}`}
          </span>
        ) : (
          <span className="badge">{t("rt.system")}</span>
        )}
      </header>

      <div className="rt-active">
        <span className="muted">{t("rt.terminalUses")}</span>
        {rt.active ? (
          <>
            <strong>{rt.active.version}</strong>
            <code className="rt-path" title={rt.active.path}>
              {tidy(rt.active.path)}
            </code>
          </>
        ) : (
          <span className="muted">{t("rt.notOnPath")}</span>
        )}
      </div>

      {rt.issue && (
        <div className="banner warn rt-issue">
          <span>
            {t(`rt.issue.${rt.issue.kind}` as const, {
              active: rt.active?.version ?? t("rt.nothing"),
              expected: rt.issue.expected,
              manager: rt.manager ?? "",
            })}
          </span>
          {rt.issue.fix.length > 0 && (
            <button className="btn" onClick={() => toggle("fix")}>
              {t("rt.fix")}
            </button>
          )}
        </div>
      )}
      {panel === "fix" && <FixPanel lang={rt.id} onClose={() => setPanel(null)} onSaved={refresh} />}

      {job && <JobLine rt={rt} job={job} autoApply={autoApply} onDismiss={onDismiss} />}

      {rt.manager ? (
        <>
          {rt.installed.length === 0 ? (
            <p className="muted small">{t("rt.noneInstalled", { manager: rt.manager })}</p>
          ) : (
            <ul className="rt-list">
              {rt.installed.map((v) => (
                <VersionRow
                  key={v.id}
                  v={v}
                  busy={busy}
                  update={updates.find((u) => u.from === v.id)}
                  onRun={(a) => run(rt.id, a, v.id)}
                  onUpgrade={onUpgrade}
                />
              ))}
            </ul>
          )}
          <div className="row tight">
            <button className="btn" disabled={busy} onClick={() => toggle("install")}>
              {t("rt.install")}
            </button>
            {rt.installed.length > 0 && (
              <button className="btn" onClick={() => toggle("project")}>
                {t("rt.project")}
              </button>
            )}
          </div>
          {panel === "install" && (
            <InstallPanel
              rt={rt}
              busy={busy}
              onInstall={(v) => {
                run(rt.id, "install", v);
                setPanel(null);
              }}
            />
          )}
          {panel === "project" && <ProjectPanel rt={rt} preferred={expected} autoApply={autoApply} />}
        </>
      ) : rt.installManager && job?.running ? null : rt.installManager ? (
        <>
          <p className="muted small">{t("rt.readOnly")}</p>
          <div className="row tight">
            <button className="btn" disabled={busy} onClick={() => toggle("mise")}>
              {t("rt.mise.offer", { name: rt.name })}
            </button>
          </div>
          {panel === "mise" && (
            <MisePanel
              busy={busy}
              onInstall={() => {
                run(rt.id, "install_manager", "");
                setPanel(null);
              }}
              onClose={() => setPanel(null)}
            />
          )}
        </>
      ) : (
        <p className="muted small">{t("rt.readOnly")}</p>
      )}
    </section>
  );
}

function VersionRow({
  v,
  busy,
  update,
  onRun,
  onUpgrade,
}: {
  v: InstalledVersion;
  busy: boolean;
  update?: RuntimeUpdate;
  onRun: (a: RuntimeAction) => void;
  onUpgrade: (u: RuntimeUpdate) => void;
}) {
  const [confirm, setConfirm] = useState(false);
  return (
    <li className={v.isDefault ? "is-default" : ""}>
      <span className="rt-version">{v.version}</span>
      {v.isDefault && <span className="badge on">{t("rt.default")}</span>}
      <span className="muted small rt-size">{size(v.bytes)}</span>
      {update && !confirm && (
        <button
          className="btn rt-update"
          disabled={busy}
          title={t(v.isDefault ? "rt.update.titleDefault" : "rt.update.title", { version: update.toVersion })}
          onClick={() => onUpgrade(update)}
        >
          ↑ {update.toVersion}
        </button>
      )}
      <span className="spacer" />
      {confirm ? (
        <>
          <span className="small">{t("rt.uninstallConfirm", { version: v.version, size: size(v.bytes) })}</span>
          <button
            className="btn danger"
            disabled={busy}
            onClick={() => {
              setConfirm(false);
              onRun("uninstall");
            }}
          >
            {t("rt.uninstall")}
          </button>
          <button className="btn" onClick={() => setConfirm(false)}>
            {t("common.cancel")}
          </button>
        </>
      ) : (
        !v.isDefault && (
          <>
            <button className="btn" disabled={busy} onClick={() => onRun("set_default")}>
              {t("rt.makeDefault")}
            </button>
            <button className="btn" disabled={busy} onClick={() => setConfirm(true)}>
              {t("rt.uninstall")}
            </button>
          </>
        )
      )}
    </li>
  );
}

function JobLine({ rt, job, autoApply, onDismiss }: { rt: Runtime; job: RuntimeJob; autoApply: boolean; onDismiss: () => void }) {
  // The UI passes ids (`cpython-3.13.14-linux-x86_64-gnu`); show the version people know.
  const shown = rt.installed.find((i) => i.id === job.version)?.version ?? job.version.replace(/^v/, "");
  const label = { name: rt.name, version: shown };
  const manager = job.action === "install_manager";
  const from = job.from ? (rt.installed.find((i) => i.id === job.from)?.version ?? job.from.replace(/^v/, "")) : "";
  if (job.running) {
    return (
      <div className="rt-job">
        <div className="rt-job-head">
          <span>{t(`rt.job.${job.action}` as const, label)}</span>
          {job.percent !== null && <span className="muted">{Math.round(job.percent)}%</span>}
          <span className="spacer" />
          <button className="btn" onClick={() => void api.runtimesCancel()}>
            {t("common.cancel")}
          </button>
        </div>
        <div className={`rt-progress ${job.percent === null ? "indeterminate" : ""}`}>
          <div style={job.percent === null ? undefined : { width: `${job.percent}%` }} />
        </div>
        {job.line && <code className="rt-job-line">{job.line}</code>}
      </div>
    );
  }
  return (
    <div className={`banner ${job.error ? "error" : "info"} rt-job-done`}>
      <span>
        {job.error ? (
          <>
            {t("rt.job.failed")}: <code className="rt-job-line">{job.error}</code>
          </>
        ) : job.cancelled ? (
          t("rt.job.cancelled")
        ) : manager ? (
          t("rt.job.doneManager", label)
        ) : job.action === "upgrade" ? (
          t("rt.job.doneUpgrade", { ...label, from })
        ) : (
          <>
            {t("rt.job.done", label)}
            {job.action === "set_default" && appliedHint(rt, autoApply) && (
              <>
                <br />
                <span className="small">{appliedHint(rt, autoApply)}</span>
              </>
            )}
          </>
        )}
      </span>
      <button className="btn" onClick={onDismiss}>
        {t("common.close")}
      </button>
    </div>
  );
}

type ListMode = "lines" | "lts" | "all";

function InstallPanel({ rt, busy, onInstall }: { rt: Runtime; busy: boolean; onInstall: (version: string) => void }) {
  const [list, setList] = useState<AvailableList | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState("");
  const [custom, setCustom] = useState("");
  const [mode, setMode] = useState<ListMode>("lines");

  useEffect(() => {
    void api.runtimesAvailable(rt.id).then(setList, (e) => setError(errorMessage(e)));
  }, [rt.id]);

  const all = list?.versions ?? [];
  const hasLts = all.some((v) => v.tag?.startsWith("LTS"));
  const hasLines = all.some((v) => v.line);
  const shown = useMemo(() => {
    const q = filter.trim().replace(/^v/, "");
    // Typing searches everything; the chips choose what shows without typing.
    if (q) return all.filter((v) => v.version.startsWith(q) || (v.tag ?? "").toLowerCase().includes(q.toLowerCase()));
    if (mode === "lts" && hasLts) return all.filter((v) => v.tag?.startsWith("LTS"));
    if (mode === "lines" && hasLines) {
      // Newest first, so the first of each line is its newest release.
      const seen = new Set<string>();
      return all.filter((v) => {
        if (!v.line || v.tag === "pre-release" || seen.has(v.line)) return false;
        seen.add(v.line);
        return true;
      });
    }
    return all;
  }, [all, filter, mode, hasLts, hasLines]);

  const chips: { id: ListMode; label: string }[] = [
    ...(hasLines ? [{ id: "lines" as const, label: t("rt.pick.lines") }] : []),
    ...(hasLts ? [{ id: "lts" as const, label: t("rt.pick.lts") }] : []),
    { id: "all", label: t("rt.pick.all", { n: all.length }) },
  ];

  return (
    <div className="rt-panel">
      <h4>{t("rt.pick.title", { name: rt.name })}</h4>
      {rt.freeInput && (
        <form
          className="row tight"
          onSubmit={(e) => {
            e.preventDefault();
            if (custom.trim()) onInstall(custom.trim());
          }}
        >
          <input className="rt-input" value={custom} onChange={(e) => setCustom(e.target.value)} placeholder={t("rt.pick.custom")} spellCheck={false} />
          <button className="btn primary" type="submit" disabled={busy || !custom.trim()}>
            {t("rt.pick.install")}
          </button>
        </form>
      )}
      {error && <div className="banner error">{error}</div>}
      {!list && !error && <p className="muted small">{t("rt.pick.loading")}</p>}
      {list && (
        <>
          {list.stale && <p className="muted small">{t("rt.pick.stale", { time: relativeTime(list.fetchedAt) })}</p>}
          {all.length > 8 && (
            <div className="row tight rt-filters">
              {chips.length > 1 &&
                chips.map((c) => (
                  <button key={c.id} className={`chip ${mode === c.id && !filter ? "on" : ""}`} onClick={() => setMode(c.id)}>
                    {c.label}
                  </button>
                ))}
              <input className="rt-input" value={filter} onChange={(e) => setFilter(e.target.value)} placeholder={t("rt.pick.filter")} spellCheck={false} />
            </div>
          )}
          <ul className="rt-avail">
            {shown.slice(0, LIST_LIMIT).map((v) => (
              <li key={v.id}>
                <span className="rt-version">{v.version}</span>
                {v.tag && <span className="badge">{v.tag}</span>}
                <span className="spacer" />
                {v.installed ? (
                  <span className="muted small">{t("rt.pick.installed")}</span>
                ) : (
                  <button className="btn" disabled={busy} onClick={() => onInstall(v.id)}>
                    {t("rt.pick.install")}
                  </button>
                )}
              </li>
            ))}
          </ul>
          {shown.length > LIST_LIMIT && <p className="muted small">{t("rt.pick.more", { n: LIST_LIMIT, total: shown.length })}</p>}
        </>
      )}
    </div>
  );
}

function ProjectPanel({ rt, preferred, autoApply }: { rt: Runtime; preferred?: string; autoApply: boolean }) {
  const [dir, setDir] = useState<string | null>(null);
  const [existing, setExisting] = useState<string | null>(null);
  const [version, setVersion] = useState(rt.installed.find((i) => i.version === preferred)?.id ?? rt.installed[0]?.id ?? "");
  const [done, setDone] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const file = rt.projectFile ?? "";

  const choose = async () => {
    setError(null);
    setDone(null);
    try {
      const picked = await api.runtimesPickFolder();
      if (!picked) return;
      setDir(picked);
      setExisting((await api.runtimesProjectGet(picked, rt.id)).content);
    } catch (e) {
      setError(errorMessage(e));
    }
  };

  return (
    <div className="rt-panel">
      <h4>{t("rt.project.title")}</h4>
      <p className="muted small">
        {t("rt.project.intro", { file, manager: rt.manager ?? "" })}
        {" "}
        {rt.manager === "nvm" && !autoApply ? t("rt.project.nvmUse") : t("rt.project.auto", { manager: rt.manager ?? "" })}
      </p>
      <div className="row tight">
        <button className="btn" onClick={() => void choose()}>
          {t("rt.project.choose")}
        </button>
        {dir && <code className="rt-path">{tidy(dir)}</code>}
      </div>
      {dir && (
        <>
          {existing !== null && (
            <>
              <p className="small">{t("rt.project.existing", { file })}</p>
              <pre className="rt-pre">{existing}</pre>
            </>
          )}
          <div className="row tight">
            <label className="rt-select">
              <span className="muted small">{t("rt.project.version")}</span>
              <select value={version} onChange={(e) => setVersion(e.target.value)}>
                {rt.installed.map((i) => (
                  <option key={i.id} value={i.id}>
                    {i.version}
                  </option>
                ))}
              </select>
            </label>
            <button
              className="btn primary"
              disabled={!version}
              onClick={() =>
                void api.runtimesProjectSet(dir, rt.id, version).then(
                  (path) => {
                    setDone(path);
                    setExisting(null);
                  },
                  (e) => setError(errorMessage(e)),
                )
              }
            >
              {t("rt.project.write", { file })}
            </button>
          </div>
        </>
      )}
      {done && <div className="banner info">{t("rt.project.done", { file: tidy(done) })}</div>}
      {error && <div className="banner error">{error}</div>}
    </div>
  );
}

function MisePanel({ busy, onInstall, onClose }: { busy: boolean; onInstall: () => void; onClose: () => void }) {
  return (
    <div className="rt-panel">
      <h4>{t("rt.mise.title")}</h4>
      <p className="muted small">{t("rt.mise.intro", { dir: "~/.local/bin" })}</p>
      <div className="row tight">
        <button className="btn primary" disabled={busy} onClick={onInstall}>
          {t("rt.mise.install")}
        </button>
        <button className="btn" onClick={onClose}>
          {t("common.cancel")}
        </button>
      </div>
    </div>
  );
}

/** Where a new default takes effect, for the line under "Done". */
function appliedHint(rt: Runtime, autoApply: boolean): string | null {
  // rustup's proxies stay at one path and read the default at every run.
  if (rt.manager === "rustup") return t("rt.applied.now");
  if (autoApply) return t("rt.applied.next");
  if (rt.manager === "nvm") return t("rt.applied.newOnly");
  if (rt.manager === "uv" || rt.manager === "mise") return t("rt.applied.hashR");
  return null;
}

/** "Apply right away in open terminals": QuickDesk's prompt hook, for every language. */
function AutoApply({ state, onChanged }: { state: AutoApplyState; onChanged: () => void }) {
  const [confirm, setConfirm] = useState(false);
  const [done, setDone] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const set = (on: boolean) =>
    api.runtimesSetAutoApply(on).then(
      () => {
        setConfirm(false);
        setDone(on);
        onChanged();
      },
      (e) => setError(errorMessage(e)),
    );
  return (
    <div className="rt-auto">
      <label className="switch">
        <input
          type="checkbox"
          checked={state.on || confirm}
          onChange={(e) => {
            setError(null);
            setDone(false);
            if (e.target.checked) setConfirm(true);
            else if (confirm) setConfirm(false);
            else void set(false);
          }}
        />
        {t("rt.auto.label")}
      </label>
      <p className="muted small">{t("rt.auto.hint")}</p>
      {confirm && (
        <div className="rt-panel">
          <h4>{t("rt.auto.title", { file: tidy(state.file) })}</h4>
          <p className="muted small">{t("rt.auto.intro", { script: tidy(state.script) })}</p>
          <pre className="rt-pre">
            <span className="rt-added">{state.line}</span>
          </pre>
          <div className="row tight">
            <button className="btn primary" onClick={() => void set(true)}>
              {t("rt.auto.apply")}
            </button>
            <button className="btn" onClick={() => setConfirm(false)}>
              {t("common.cancel")}
            </button>
          </div>
        </div>
      )}
      {done && <div className="banner info">{t("rt.auto.done")}</div>}
      {error && <div className="banner error">{error}</div>}
    </div>
  );
}

function FixPanel({ lang, onClose, onSaved }: { lang: string; onClose: () => void; onSaved: () => void }) {
  const [fix, setFix] = useState<ShellFix | null>(null);
  const [saved, setSaved] = useState<{ backup: string | null } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void api.runtimesShellFix(lang).then(setFix, (e) => setError(errorMessage(e)));
  }, [lang]);

  if (error) return <div className="banner error">{error}</div>;
  if (!fix) return null;
  return (
    <div className="rt-panel">
      <h4>{t("rt.fix.title", { file: tidy(fix.file) })}</h4>
      <p className="muted small">{t("rt.fix.intro")}</p>
      <pre className="rt-pre">
        {"# >>> quickdesk >>>\n"}
        {fix.lines.map((l) => (
          <span key={l} className={fix.current.includes(l) ? "" : "rt-added"}>
            {l + "\n"}
          </span>
        ))}
        {"# <<< quickdesk <<<"}
      </pre>
      {saved ? (
        <div className="banner info">
          <span>
            {t("rt.fix.done")}
            {saved.backup && (
              <>
                <br />
                <span className="small">{t("rt.fix.backup", { file: tidy(saved.backup) })}</span>
              </>
            )}
          </span>
          <button className="btn" onClick={onClose}>
            {t("common.close")}
          </button>
        </div>
      ) : (
        <div className="row tight">
          <button
            className="btn primary"
            onClick={() =>
              void api.runtimesShellApply(lang).then(
                (r) => {
                  setSaved({ backup: r.backup });
                  onSaved();
                },
                (e) => setError(errorMessage(e)),
              )
            }
          >
            {t("rt.fix.apply")}
          </button>
          <button className="btn" onClick={onClose}>
            {t("common.cancel")}
          </button>
        </div>
      )}
      {fix.current.length > 0 && !saved && (
        <button
          className="btn link small"
          onClick={() =>
            void api.runtimesShellUndo().then(
              () => {
                onSaved();
                onClose();
              },
              (e) => setError(errorMessage(e)),
            )
          }
        >
          {t("rt.fix.undo", { file: tidy(fix.file) })}
        </button>
      )}
    </div>
  );
}
