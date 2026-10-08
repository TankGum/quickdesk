import { lazy, Suspense, useCallback, useEffect, useRef, useState } from "react";

import { useI18n } from "../../shared/i18n";
import { api, errorMessage, Note, useBackendEvent } from "../../shared/ipc";
import { absoluteTime, relativeTime } from "../../shared/time";
import { ConfirmDialog } from "../../shared/ConfirmDialog";
import { markdownToText } from "./markdown";

// The rich editor is large; load it only when a note is opened.
const NoteDialog = lazy(() => import("./NoteDialog").then((m) => ({ default: m.NoteDialog })));

const UNDO_MS = 5000;

/** What the list shows as a note's heading: its title, or its first line. */
export function heading(n: Note): { text: string; preview: string } {
  const lines = markdownToText(n.body).split("\n").filter((l) => l.trim());
  if (n.title.trim()) return { text: n.title, preview: lines.join("\n") };
  return { text: lines[0] ?? "", preview: lines.slice(1).join("\n") };
}

/** `focusSignal` changes each time the main window is brought up. */
export function NotesTab({ focusSignal }: { focusSignal: number }) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [notes, setNotes] = useState<Note[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [undo, setUndo] = useState<Note | null>(null);
  /** undefined = closed, null = new note, Note = editing it. */
  const [editing, setEditing] = useState<Note | null | undefined>(undefined);
  const [confirmDelete, setConfirmDelete] = useState<Note | null>(null);
  const search = useRef<HTMLInputElement>(null);

  const refresh = useCallback(() => {
    const q = query.trim();
    (q ? api.notesSearch(q) : api.notesList())
      .then((n) => {
        setNotes(n);
        setError(null);
      })
      .catch((e) => setError(errorMessage(e)));
  }, [query]);

  // Debounce typing; refresh immediately on backend changes.
  useEffect(() => {
    const id = setTimeout(refresh, 80);
    return () => clearTimeout(id);
  }, [refresh]);
  useBackendEvent("notes://changed", refresh);

  // Opening the manager focuses search; Ctrl+N writes, clicking a note edits.
  useEffect(() => {
    if (editing === undefined) search.current?.focus();
  }, [focusSignal, editing]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey) || editing !== undefined || confirmDelete) return;
      const k = e.key.toLowerCase();
      if (k === "f") {
        e.preventDefault();
        search.current?.focus();
        search.current?.select();
      } else if (k === "n") {
        e.preventDefault();
        setEditing(null);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [editing, confirmDelete]);

  useEffect(() => {
    if (!undo) return;
    const id = setTimeout(() => setUndo(null), UNDO_MS);
    return () => clearTimeout(id);
  }, [undo]);

  const run = (p: Promise<unknown>) => p.catch((e) => setError(errorMessage(e)));

  const remove = (n: Note) => {
    setConfirmDelete(null);
    void run(api.notesDelete(n.id).then(() => setUndo(n)));
  };

  return (
    <div className="notes">
      <div className="toolbar">
        <input
          ref={search}
          className="search"
          placeholder={t("notes.search")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") setQuery("");
            if (e.key === "Enter" && notes[0]) setEditing(notes[0]);
          }}
        />
        <button className="btn primary" title={t("notes.newHint")} onClick={() => setEditing(null)}>
          {t("notes.new")}
        </button>
      </div>
      {error && <div className="banner error">{error}</div>}
      {notes.length === 0 ? (
        <div className="empty">{query ? t("notes.noMatches") : t("notes.empty")}</div>
      ) : (
        <ul className="note-list">
          {notes.map((n) => {
            const h = heading(n);
            const preview = h.preview;
            return (
              <li key={n.id} className={`note ${n.pinned ? "pinned" : ""}`}>
                <button
                  className={`icon pin ${n.pinned ? "on" : ""}`}
                  title={n.pinned ? t("common.unpin") : t("common.pin")}
                  onClick={() => void run(api.notesUpdate(n.id, { pinned: !n.pinned }))}
                >
                  {n.pinned ? "★" : "☆"}
                </button>
                <button className="note-main note-open" onClick={() => setEditing(n)}>
                  <div className="note-heading">{h.text || t("notes.untitled")}</div>
                  {preview && <div className="note-preview">{preview}</div>}
                  <div className="note-meta">
                    {n.conflictOf && (
                      <span className="badge warn" title={t("notes.conflictHint")}>
                        {t("notes.conflict")}
                      </span>
                    )}
                    <span title={absoluteTime(n.updatedAt)}>{relativeTime(n.updatedAt)}</span>
                  </div>
                </button>
                <button className="icon delete" title={t("common.delete")} onClick={() => setConfirmDelete(n)}>
                  🗑️
                </button>
              </li>
            );
          })}
        </ul>
      )}
      {editing !== undefined && (
        <Suspense fallback={<div className="dialog-backdrop" />}>
          <NoteDialog
            key={editing?.id ?? "new"}
            note={editing}
            onClose={() => setEditing(undefined)}
            onDeleted={(n) => setUndo(n)}
          />
        </Suspense>
      )}
      {confirmDelete && (
        <ConfirmDialog
          title={t("notes.deleteTitle")}
          message={t("notes.deleteBody", { name: heading(confirmDelete).text || t("notes.untitled") })}
          confirmLabel={t("common.delete")}
          onConfirm={() => remove(confirmDelete)}
          onCancel={() => setConfirmDelete(null)}
        />
      )}
      {undo && (
        <div className="toast">
          {t("notes.deleted")}
          <button
            onClick={() => {
              void run(api.notesRestore(undo.id));
              setUndo(null);
            }}
          >
            {t("common.undo")}
          </button>
        </div>
      )}
    </div>
  );
}
