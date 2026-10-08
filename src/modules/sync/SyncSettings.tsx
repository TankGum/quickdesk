import { useState } from "react";

import { api, errorMessage, S3Config } from "../../shared/ipc";
import { relativeTime } from "../../shared/time";
import { useSyncStatus } from "./useSyncStatus";

const EMPTY: S3Config = { endpoint: "", bucket: "", region: "auto", accessKeyId: "", prefix: "quickdesk" };

/** Connect storage → create keys (first device) or unlock (others) → syncing. */
export function SyncSettings() {
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
        <dt>Status</dt>
        <dd>
          <span className={`sync-state ${status.state}`}>{status.state}</span>
          {status.lastSyncAt && <span className="muted"> · last sync {relativeTime(status.lastSyncAt)}</span>}
        </dd>
        {status.lastError && (
          <>
            <dt>Last error</dt>
            <dd className="error-text">{status.lastError}</dd>
          </>
        )}
        {r && (
          <>
            <dt>Last round</dt>
            <dd>
              ↓ {r.pulled} · ↑ {r.pushed}
              {r.conflicts > 0 && ` · ${r.conflicts} conflict(s) kept as copies`}
            </dd>
          </>
        )}
        <dt>Storage</dt>
        <dd>
          <code>{status.config?.endpoint}</code> / <code>{status.config?.bucket}</code>
          {status.config?.prefix && <> / <code>{status.config.prefix}</code></>}
        </dd>
      </dl>
      {error && <div className="banner error">{error}</div>}
      <div className="row">
        <button className="btn" disabled={busy} onClick={() => void api.syncNow()}>
          Sync now
        </button>
        <DisconnectButton onConfirm={() => act(api.syncDisconnect)} busy={busy} />
      </div>
      <p className="muted small">
        Notes are encrypted on this device before upload; the storage provider only sees ciphertext. Clipboard history never syncs.
      </p>
    </div>
  );
}

function ConnectForm({ busy, error, onConnect }: { busy: boolean; error: string | null; onConnect: (c: S3Config, secret: string) => void }) {
  const [cfg, setCfg] = useState<S3Config>(EMPTY);
  const [secret, setSecret] = useState("");
  const field = (key: keyof S3Config, label: string, placeholder: string, hint?: string) => (
    <label className="field">
      <span>{label}</span>
      <input value={cfg[key]} placeholder={placeholder} onChange={(e) => setCfg({ ...cfg, [key]: e.target.value })} spellCheck={false} />
      {hint && <small className="muted">{hint}</small>}
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
      <p className="muted">
        Sync Quick Notes through any S3-compatible bucket you own: Cloudflare R2 (recommended, no egress fees), AWS S3 or MinIO.
        Use an API token limited to this one bucket.
      </p>
      {field("endpoint", "Endpoint", "https://<account-id>.r2.cloudflarestorage.com", "R2: Dashboard → R2 → S3 API. AWS: https://s3.<region>.amazonaws.com")}
      {field("bucket", "Bucket", "my-quickdesk")}
      {field("region", "Region", "auto", "Use “auto” for R2")}
      {field("accessKeyId", "Access key ID", "")}
      <label className="field">
        <span>Secret access key</span>
        <input type="password" value={secret} onChange={(e) => setSecret(e.target.value)} autoComplete="off" />
        <small className="muted">Stored in the OS keyring, not in the app database.</small>
      </label>
      {field("prefix", "Folder prefix", "quickdesk", "Optional: lets several apps share a bucket")}
      {error && <div className="banner error">{error}</div>}
      <button className="btn primary" type="submit" disabled={busy}>
        {busy ? "Connecting…" : "Connect"}
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
      <h3>Set an encryption passphrase</h3>
      <p className="muted">
        This is the first device on this bucket. Choose a passphrase you will type on your other devices. It never leaves this
        machine; without it (or the recovery key shown next) synced notes cannot be decrypted.
      </p>
      <label className="field">
        <span>Passphrase (8+ characters)</span>
        <input type="password" value={p1} onChange={(e) => setP1(e.target.value)} autoFocus />
      </label>
      <label className="field">
        <span>Repeat passphrase</span>
        <input type="password" value={p2} onChange={(e) => setP2(e.target.value)} />
        {mismatch && <small className="error-text">Passphrases do not match</small>}
      </label>
      {error && <div className="banner error">{error}</div>}
      <div className="row">
        <button className="btn primary" type="submit" disabled={busy || p1.length < 8 || p1 !== p2}>
          {busy ? "Creating keys…" : "Create keys"}
        </button>
        <button className="btn" type="button" onClick={onCancel}>
          Back
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
      <h3>Unlock sync</h3>
      <p className="muted">This bucket already holds QuickDesk data. Enter the passphrase you set on your first device, or your recovery key.</p>
      <label className="field">
        <span>Passphrase or recovery key</span>
        <input type="password" value={secret} onChange={(e) => setSecret(e.target.value)} autoFocus />
      </label>
      {error && <div className="banner error">{error}</div>}
      <button className="btn primary" type="submit" disabled={busy || !secret}>
        {busy ? "Unlocking…" : "Unlock"}
      </button>
    </form>
  );
}

function RecoveryKeyNotice({ recoveryKey, onDone }: { recoveryKey: string; onDone: () => void }) {
  const [saved, setSaved] = useState(false);
  const [copied, setCopied] = useState(false);
  return (
    <div className="sync-form">
      <h3>Save your recovery key</h3>
      <p>
        If you forget your passphrase, this key is the <b>only</b> way to decrypt your synced notes. It is shown once. Store it in
        a password manager.
      </p>
      <pre className="recovery-key">{recoveryKey}</pre>
      <div className="row">
        <button className="btn" onClick={() => void api.clipboardWrite(recoveryKey).then(() => setCopied(true))}>
          {copied ? "Copied ✓" : "Copy"}
        </button>
      </div>
      <label className="switch">
        <input type="checkbox" checked={saved} onChange={(e) => setSaved(e.target.checked)} /> I have saved my recovery key
      </label>
      <button className="btn primary" disabled={!saved} onClick={onDone}>
        Done
      </button>
    </div>
  );
}

function DisconnectButton({ onConfirm, busy }: { onConfirm: () => void; busy: boolean }) {
  const [confirm, setConfirm] = useState(false);
  return confirm ? (
    <>
      <span className="muted">Stop syncing on this device? Local notes are kept.</span>
      <button className="btn danger" disabled={busy} onClick={onConfirm}>
        Disconnect
      </button>
      <button className="btn" onClick={() => setConfirm(false)}>
        Cancel
      </button>
    </>
  ) : (
    <button className="btn" onClick={() => setConfirm(true)}>
      Disconnect…
    </button>
  );
}
