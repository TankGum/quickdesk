import { useState } from "react";

import { t, useI18n } from "../../shared/i18n";
import { api, errorMessage } from "../../shared/ipc";
import { relativeTime } from "../../shared/time";
import { useSyncStatus } from "./useSyncStatus";

type Step = "start" | "enable" | "join" | "done";

/** QuickDesk Cloud: turn sync on (first device) or join with a sync code. */
export function SyncSettings() {
  useI18n();
  const status = useSyncStatus();
  const [step, setStep] = useState<Step>("start");
  const [created, setCreated] = useState<{ syncCode: string; recoveryKey: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const act = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  };
  const go = (s: Step) => {
    setError(null);
    setStep(s);
  };

  if (!status) return null;

  // Stays up after the backend has moved on: the codes are shown only here.
  if (step === "done" && created) {
    return <SavedCodes {...created} onDone={() => go("start")} />;
  }

  if (status.state === "disabled") {
    if (step === "enable") {
      return (
        <EnableForm
          busy={busy}
          error={error}
          onCancel={() => go("start")}
          onEnable={(pass) =>
            act(async () => {
              setCreated(await api.syncEnable(pass));
              setStep("done");
            })
          }
        />
      );
    }
    if (step === "join") {
      return (
        <JoinForm
          busy={busy}
          error={error}
          onCancel={() => go("start")}
          onJoin={(code, secret) => act(async () => { await api.syncJoin(code, secret); go("start"); })}
        />
      );
    }
    return (
      <div className="sync-form">
        {status.notice === "bucket_removed" && <div className="banner info">{t("sync.notice.bucketRemoved")}</div>}
        <p>{t("sync.cloud.intro")}</p>
        <div className="row">
          <button className="btn primary" onClick={() => go("enable")}>
            {t("sync.cloud.enable")}
          </button>
          <button className="btn" onClick={() => go("join")}>
            {t("sync.cloud.haveCode")}
          </button>
        </div>
        <p className="muted small">{t("sync.e2e")}</p>
      </div>
    );
  }

  if (status.state === "locked") {
    return <Unlock busy={busy} error={error} onUnlock={(s) => act(() => api.syncUnlock(s))} />;
  }

  const r = status.lastReport;
  return (
    <div className="sync-panel">
      <dl>
        <dt>{t("sync.status")}</dt>
        <dd>
          <span className={`sync-state ${status.state}`}>{t(`sync.state.${status.state}` as const)}</span>
          {status.lastSyncAt && <span className="muted">{t("sync.lastSync", { time: relativeTime(status.lastSyncAt) })}</span>}
        </dd>
        {status.lastError && (
          <>
            <dt>{t("sync.lastError")}</dt>
            <dd className="error-text">{status.lastError}</dd>
          </>
        )}
        {r && (
          <>
            <dt>{t("sync.lastRound")}</dt>
            <dd>
              ↓ {r.pulled} · ↑ {r.pushed}
              {r.conflicts > 0 && t("sync.conflicts", { n: r.conflicts })}
            </dd>
          </>
        )}
        <dt>{t("sync.cloud.code")}</dt>
        <dd>
          <ShowCode />
        </dd>
      </dl>
      {error && <div className="banner error">{error}</div>}
      <div className="row">
        <button className="btn" disabled={busy} onClick={() => void api.syncNow()}>
          {t("sync.now")}
        </button>
        <DisconnectButton busy={busy} onConfirm={(deleteCloud) => act(() => api.syncDisconnect(deleteCloud))} />
      </div>
      <p className="muted small">{t("sync.e2e")}</p>
    </div>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button className="btn" type="button" onClick={() => void api.clipboardWrite(text).then(() => setCopied(true))}>
      {copied ? t("common.copied") : t("common.copy")}
    </button>
  );
}

/** The sync code on demand, to add another device. */
function ShowCode() {
  const [code, setCode] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  if (!code) {
    return (
      <>
        <button className="btn" onClick={() => void api.syncCode().then(setCode, (e) => setError(errorMessage(e)))}>
          {t("sync.cloud.showCode")}
        </button>
        {error && <span className="error-text small"> {error}</span>}
      </>
    );
  }
  return (
    <div className="sync-code-row">
      <code className="sync-code">{code}</code>
      <CopyButton text={code} />
      <span className="muted small">{t("sync.cloud.codeHint")}</span>
    </div>
  );
}

function EnableForm({ busy, error, onEnable, onCancel }: { busy: boolean; error: string | null; onEnable: (p: string) => void; onCancel: () => void }) {
  const [p1, setP1] = useState("");
  const [p2, setP2] = useState("");
  const mismatch = p2.length > 0 && p1 !== p2;
  return (
    <form
      className="sync-form"
      onSubmit={(e) => {
        e.preventDefault();
        if (!mismatch) onEnable(p1);
      }}
    >
      <h3>{t("sync.create.title")}</h3>
      <p className="muted">{t("sync.create.intro")}</p>
      <label className="field">
        <span>{t("sync.create.pass")}</span>
        <input type="password" value={p1} onChange={(e) => setP1(e.target.value)} autoFocus />
      </label>
      <label className="field">
        <span>{t("sync.create.repeat")}</span>
        <input type="password" value={p2} onChange={(e) => setP2(e.target.value)} />
        {mismatch && <small className="error-text">{t("sync.create.mismatch")}</small>}
      </label>
      {error && <div className="banner error">{error}</div>}
      <div className="row">
        <button className="btn primary" type="submit" disabled={busy || p1.length < 8 || p1 !== p2}>
          {busy ? t("sync.cloud.enabling") : t("sync.cloud.enable")}
        </button>
        <button className="btn" type="button" onClick={onCancel}>
          {t("sync.back")}
        </button>
      </div>
    </form>
  );
}

function JoinForm({ busy, error, onJoin, onCancel }: { busy: boolean; error: string | null; onJoin: (code: string, secret: string) => void; onCancel: () => void }) {
  const [code, setCode] = useState("");
  const [secret, setSecret] = useState("");
  return (
    <form
      className="sync-form"
      onSubmit={(e) => {
        e.preventDefault();
        onJoin(code, secret);
      }}
    >
      <h3>{t("sync.join.title")}</h3>
      <p className="muted">{t("sync.join.intro")}</p>
      <label className="field">
        <span>{t("sync.cloud.code")}</span>
        <input value={code} onChange={(e) => setCode(e.target.value)} placeholder="QD1-XXXXXX-XXXXXX-…" spellCheck={false} autoFocus />
      </label>
      <label className="field">
        <span>{t("sync.unlock.label")}</span>
        <input type="password" value={secret} onChange={(e) => setSecret(e.target.value)} />
      </label>
      {error && <div className="banner error">{error}</div>}
      <div className="row">
        <button className="btn primary" type="submit" disabled={busy || !code.trim() || !secret}>
          {busy ? t("sync.join.busy") : t("sync.join.button")}
        </button>
        <button className="btn" type="button" onClick={onCancel}>
          {t("sync.back")}
        </button>
      </div>
    </form>
  );
}

function Unlock({ busy, error, onUnlock }: { busy: boolean; error: string | null; onUnlock: (s: string) => void }) {
  const [secret, setSecret] = useState("");
  return (
    <form
      className="sync-form"
      onSubmit={(e) => {
        e.preventDefault();
        onUnlock(secret);
      }}
    >
      <h3>{t("sync.unlock.title")}</h3>
      <p className="muted">{t("sync.unlock.intro")}</p>
      <label className="field">
        <span>{t("sync.unlock.label")}</span>
        <input type="password" value={secret} onChange={(e) => setSecret(e.target.value)} autoFocus />
      </label>
      {error && <div className="banner error">{error}</div>}
      <button className="btn primary" type="submit" disabled={busy || !secret}>
        {busy ? t("sync.unlock.busy") : t("sync.unlock.button")}
      </button>
    </form>
  );
}

/** After turning sync on: the sync code (for other devices) and the recovery key. */
function SavedCodes({ syncCode, recoveryKey, onDone }: { syncCode: string; recoveryKey: string; onDone: () => void }) {
  const [saved, setSaved] = useState(false);
  return (
    <div className="sync-form">
      <h3>{t("sync.done.title")}</h3>
      <p>{t("sync.done.code")}</p>
      <div className="sync-code-row">
        <code className="sync-code">{syncCode}</code>
        <CopyButton text={syncCode} />
      </div>
      <p>{t("sync.recovery.intro")}</p>
      <pre className="recovery-key">{recoveryKey}</pre>
      <div className="row">
        <CopyButton text={recoveryKey} />
      </div>
      <label className="switch">
        <input type="checkbox" checked={saved} onChange={(e) => setSaved(e.target.checked)} /> {t("sync.recovery.saved")}
      </label>
      <button className="btn primary" disabled={!saved} onClick={onDone}>
        {t("sync.recovery.done")}
      </button>
    </div>
  );
}

function DisconnectButton({ onConfirm, busy }: { onConfirm: (deleteCloud: boolean) => void; busy: boolean }) {
  const [confirm, setConfirm] = useState(false);
  const [deleteCloud, setDeleteCloud] = useState(false);
  return confirm ? (
    <div className="disconnect-box">
      <span className="muted">{t("sync.disconnect.confirm")}</span>
      <label className="switch">
        <input type="checkbox" checked={deleteCloud} onChange={(e) => setDeleteCloud(e.target.checked)} /> {t("sync.disconnect.deleteCloud")}
      </label>
      <div className="row tight">
        <button className="btn danger" disabled={busy} onClick={() => onConfirm(deleteCloud)}>
          {deleteCloud ? t("sync.disconnect.deleteButton") : t("sync.disconnect.button")}
        </button>
        <button className="btn" onClick={() => setConfirm(false)}>
          {t("common.cancel")}
        </button>
      </div>
    </div>
  ) : (
    <button className="btn" onClick={() => setConfirm(true)}>
      {t("sync.disconnect")}
    </button>
  );
}
