import { useEffect, useRef, useState } from "react";

import { useI18n } from "../../shared/i18n";
import { api, errorMessage, Note } from "../../shared/ipc";
import { absoluteTime, relativeTime } from "../../shared/time";

/** `note` = edit that note; `null` = write a new one. */
export function NoteDialog({ note, onClose, onDeleted }: { note: Note | null; onClose: () => void; onDeleted: (n: Note) => void }) {
  const { t } = useI18n();
  const [title, setTitle] = useState(note?.title ?? "");
  const [body, setBody] = useState(note?.body ?? "");
  const [pinned, setPinned] = useState(note?.pinned ?? false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmClose, setConfirmClose] = useState(false);
  const titleRef = useRef<HTMLInputElement>(null);
  const bodyRef = useRef<HTMLTextAreaElement>(null);

  const dirty = title !== (note?.title ?? "") || body !== (note?.body ?? "") || pinned !== (note?.pinned ?? false);
  const empty = !title.trim() && !body.trim();

  // New notes start in the title; existing ones at the end of the text.
  useEffect(() => {
    if (!note) {
      titleRef.current?.focus();
    } else {
      const el = bodyRef.current;
      el?.focus();
      el?.setSelectionRange(el.value.length, el.value.length);
    }
  }, [note]);

  const save = async () => {
    if (saving) return;
    if (empty) {
      onClose();
      return;
    }
    setSaving(true);
    setError(null);
    try {
      if (note) await api.notesUpdate(note.id, { title, body, pinned });
      else {
        const created = await api.notesCreate(body, title);
        if (pinned) await api.notesUpdate(created.id, { pinned });
      }
      onClose();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setSaving(false);
    }
  };

  const requestClose = () => {
    if (dirty && !empty) setConfirmClose(true);
    else onClose();
  };

  const remove = async () => {
    if (!note) return onClose();
    try {
      await api.notesDelete(note.id);
      onDeleted(note);
      onClose();
    } catch (e) {
      setError(errorMessage(e));
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && (e.key.toLowerCase() === "s" || e.key === "Enter")) {
      e.preventDefault();
      void save();
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      if (confirmClose) setConfirmClose(false);
      else requestClose();
    }
  };

  return (
    <div className="dialog-backdrop" onMouseDown={(e) => e.target === e.currentTarget && requestClose()}>
      <div className="dialog note-dialog" role="dialog" aria-modal="true" aria-label={note ? t("notes.editTitle") : t("notes.newTitle")} onKeyDown={onKeyDown}>
        <div className="dialog-head">
          <input
            ref={titleRef}
            className="note-title-input"
            placeholder={t("notes.titlePlaceholder")}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.ctrlKey && !e.metaKey) {
                e.preventDefault();
                bodyRef.current?.focus();
              }
            }}
          />
          <button className={`icon pin ${pinned ? "on" : ""}`} title={pinned ? t("common.unpin") : t("common.pin")} onClick={() => setPinned((p) => !p)}>
            {pinned ? "★" : "☆"}
          </button>
        </div>
        <textarea
          ref={bodyRef}
          className="note-body-input"
          placeholder={t("notes.bodyPlaceholder")}
          value={body}
          onChange={(e) => setBody(e.target.value)}
        />
        {note?.conflictOf && (
          <div className="banner warn-text small" title={t("notes.conflictHint")}>
            {t("notes.conflict")}: {t("notes.conflictHint")}
          </div>
        )}
        {error && <div className="banner error">{error}</div>}
        <div className="dialog-foot">
          {confirmClose ? (
            <>
              <span className="muted">{t("notes.unsaved")}</span>
              <span className="spacer" />
              <button className="btn" onClick={() => setConfirmClose(false)}>
                {t("notes.keepEditing")}
              </button>
              <button className="btn danger" onClick={onClose}>
                {t("notes.discard")}
              </button>
              <button className="btn primary" disabled={saving} onClick={() => void save()}>
                {t("notes.save")}
              </button>
            </>
          ) : (
            <>
              {note && (
                <button className="btn danger" onClick={() => void remove()}>
                  {t("common.delete")}
                </button>
              )}
              <span className="muted small" title={note ? t("notes.created", { time: absoluteTime(note.createdAt) }) : undefined}>
                {note ? t("notes.updated", { time: relativeTime(note.updatedAt) }) : t("notes.editorHint")}
              </span>
              <span className="spacer" />
              <button className="btn" onClick={requestClose}>
                {t("notes.close")}
              </button>
              <button className="btn primary" disabled={saving || (!dirty && !!note)} onClick={() => void save()}>
                {saving ? t("notes.saving") : t("notes.save")}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
