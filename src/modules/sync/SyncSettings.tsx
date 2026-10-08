import { useState } from "react";

import { Key, t, useI18n } from "../../shared/i18n";
import { api, errorMessage, S3Config } from "../../shared/ipc";
import { relativeTime } from "../../shared/time";
import { useSyncStatus } from "./useSyncStatus";

const EMPTY: S3Config = { endpoint: "", bucket: "", region: "auto", accessKeyId: "", prefix: "quickdesk" };

/** Connect storage → create keys (first device) or unlock (others) → syncing. */
export function SyncSettings() {
  useI18n();
  const status = useSyncStatus();
  const [step, setStep] = useState<"form" | "create" | "unlock" | "recovery">("form");
  const [recoveryKey, setRecoveryKey] = useState("");
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

  if (!status) return null;

  // The recovery key screen must stay up even though the backend already moved on.
  if (step === "recovery") {
    return <RecoveryKeyNotice recoveryKey={recoveryKey} onDone={() => setStep("form")} />;
  }

  if (status.state === "disabled" || step === "create" || step === "unlock") {
    if (step === "create") {
      return (
        <CreateKeys
          busy={busy}
          error={error}
          onCancel={() => setStep("form")}
          onCreate={(pass) =>
            act(async () => {
              setRecoveryKey(await api.syncCreate(pass));
              setStep("recovery");
            })
          }
        />
      );
    }
    if (step === "unlock") {
      return <Unlock busy={busy} error={error} onUnlock={(s) => act(async () => { await api.syncUnlock(s); setStep("form"); })} />;
    }
    return (
      <ConnectForm
        busy={busy}
        error={error}
        onConnect={(cfg, secret) =>
          act(async () => {
            const { initialized } = await api.syncConnect(cfg, secret);
            setStep(initialized ? "unlock" : "create");
          })
        }
      />
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
        <dt>{t("sync.storage")}</dt>
        <dd>
          <code>{status.config?.endpoint}</code> / <code>{status.config?.bucket}</code>
          {status.config?.prefix && <> / <code>{status.config.prefix}</code></>}
        </dd>
      </dl>
      {error && <div className="banner error">{error}</div>}
      <div className="row">
        <button className="btn" disabled={busy} onClick={() => void api.syncNow()}>
          {t("sync.now")}
        </button>
        <DisconnectButton onConfirm={() => act(api.syncDisconnect)} busy={busy} />
      </div>
      <p className="muted small">{t("sync.e2e")}</p>
    </div>
  );
}

function ConnectForm({ busy, error, onConnect }: { busy: boolean; error: string | null; onConnect: (c: S3Config, secret: string) => void }) {
  const [cfg, setCfg] = useState<S3Config>(EMPTY);
  const [secret, setSecret] = useState("");
  const field = (key: keyof S3Config, label: Key, placeholder: string, hint?: Key) => (
    <label className="field">
      <span>{t(label)}</span>
      <input value={cfg[key]} placeholder={placeholder} onChange={(e) => setCfg({ ...cfg, [key]: e.target.value })} spellCheck={false} />
      {hint && <small className="muted">{t(hint)}</small>}
    </label>
  );
  return (
    <form
      className="sync-form"
      onSubmit={(e) => {
        e.preventDefault();
        onConnect(cfg, secret);
      }}
    >
      <p className="muted">{t("sync.intro")}</p>
      {field("endpoint", "sync.endpoint", "https://<account-id>.r2.cloudflarestorage.com", "sync.endpointHint")}
      {field("bucket", "sync.bucket", "my-quickdesk")}
      {field("region", "sync.region", "auto", "sync.regionHint")}
      {field("accessKeyId", "sync.accessKey", "")}
      <label className="field">
        <span>{t("sync.secret")}</span>
        <input type="password" value={secret} onChange={(e) => setSecret(e.target.value)} autoComplete="off" />
        <small className="muted">{t("sync.secretHint")}</small>
      </label>
      {field("prefix", "sync.prefix", "quickdesk", "sync.prefixHint")}
      {error && <div className="banner error">{error}</div>}
      <button className="btn primary" type="submit" disabled={busy}>
        {busy ? t("sync.connecting") : t("sync.connect")}
      </button>
    </form>
  );
}

function CreateKeys({ busy, error, onCreate, onCancel }: { busy: boolean; error: string | null; onCreate: (p: string) => void; onCancel: () => void }) {
  const [p1, setP1] = useState("");
  const [p2, setP2] = useState("");
  const mismatch = p2.length > 0 && p1 !== p2;
  return (
    <form
      className="sync-form"
      onSubmit={(e) => {
        e.preventDefault();
        if (!mismatch) onCreate(p1);
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
          {busy ? t("sync.create.busy") : t("sync.create.button")}
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

function RecoveryKeyNotice({ recoveryKey, onDone }: { recoveryKey: string; onDone: () => void }) {
  const [saved, setSaved] = useState(false);
  const [copied, setCopied] = useState(false);
  return (
    <div className="sync-form">
      <h3>{t("sync.recovery.title")}</h3>
      <p>{t("sync.recovery.intro")}</p>
      <pre className="recovery-key">{recoveryKey}</pre>
      <div className="row">
        <button className="btn" onClick={() => void api.clipboardWrite(recoveryKey).then(() => setCopied(true))}>
          {copied ? t("common.copied") : t("common.copy")}
        </button>
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

function DisconnectButton({ onConfirm, busy }: { onConfirm: () => void; busy: boolean }) {
  const [confirm, setConfirm] = useState(false);
  return confirm ? (
    <>
      <span className="muted">{t("sync.disconnect.confirm")}</span>
      <button className="btn danger" disabled={busy} onClick={onConfirm}>
        {t("sync.disconnect.button")}
      </button>
      <button className="btn" onClick={() => setConfirm(false)}>
        {t("common.cancel")}
      </button>
    </>
  ) : (
    <button className="btn" onClick={() => setConfirm(true)}>
      {t("sync.disconnect")}
    </button>
  );
}
