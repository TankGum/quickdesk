import { useEffect, useRef, useState } from "react";

import { useI18n } from "../../shared/i18n";
import { api, errorMessage, Note } from "../../shared/ipc";
import { absoluteTime, relativeTime } from "../../shared/time";
import { ConfirmDialog } from "../../shared/ConfirmDialog";
import { NoteEditor } from "./NoteEditor";

/** `note` = edit that note; `null` = write a new one. */
export function NoteDialog({ note, onClose, onDeleted }: { note: Note | null; onClose: () => void; onDeleted: (n: Note) => void }) {
  const { t } = useI18n();
  const [title, setTitle] = useState(note?.title ?? "");
  const [body, setBody] = useState(note?.body ?? "");
  const [pinned, setPinned] = useState(note?.pinned ?? false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmClose, setConfirmClose] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  // Only real edits count: re-serializing an untouched note can differ
  // byte-for-byte (Markdown escaping) and must not look like a change.
  const [bodyTouched, setBodyTouched] = useState(false);
  const titleRef = useRef<HTMLInputElement>(null);
  const editorRef = useRef<{ commands: { focus: (p?: "start" | "end") => unknown } } | null>(null);

  const dirty = title !== (note?.title ?? "") || bodyTouched || pinned !== (note?.pinned ?? false);
  const empty = !title.trim() && !body.trim();

  // New notes start in the title; existing ones at the end of the text.
  useEffect(() => {
    if (!note) titleRef.current?.focus();
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
      if (note) await api.notesUpdate(note.id, { title, pinned, ...(bodyTouched ? { body } : {}) });
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
    setConfirmDelete(false);
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
    // Ctrl+Shift+S is strikethrough in the editor, so only plain Ctrl+S saves.
    if (mod && !e.shiftKey && (e.key.toLowerCase() === "s" || e.key === "Enter")) {
      e.preventDefault();
      void save();
    } else if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      if (confirmDelete) setConfirmDelete(false);
      else if (confirmClose) setConfirmClose(false);
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
                editorRef.current?.commands.focus("start");
              }
            }}
          />
          <button className={`icon pin ${pinned ? "on" : ""}`} title={pinned ? t("common.unpin") : t("common.pin")} onClick={() => setPinned((p) => !p)}>
            {pinned ? "★" : "☆"}
          </button>
        </div>
        <NoteEditor
          markdown={note?.body ?? ""}
          autofocus={!!note}
          onReady={(ed) => (editorRef.current = ed)}
          onChange={(md) => {
            setBody(md);
            setBodyTouched(true);
          }}
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
                <button className="btn danger" onClick={() => setConfirmDelete(true)}>
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
      {confirmDelete && note && (
        <ConfirmDialog
          title={t("notes.deleteTitle")}
          message={t("notes.deleteBody", { name: note.title || body.split("\n")[0] || t("notes.untitled") })}
          confirmLabel={t("common.delete")}
          onConfirm={() => void remove()}
          onCancel={() => setConfirmDelete(false)}
        />
      )}
    </div>
  );
}
