import { forwardRef, useCallback, useEffect, useRef, useState } from "react";

import { api, errorMessage, Note, useBackendEvent } from "../../shared/ipc";
import { absoluteTime, relativeTime } from "../../shared/time";

const UNDO_MS = 5000;

/** `focusSignal` changes each time the main window is brought up. */
export function NotesTab({ focusSignal }: { focusSignal: number }) {
  const [query, setQuery] = useState("");
  const [notes, setNotes] = useState<Note[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [undo, setUndo] = useState<Note | null>(null);
  const search = useRef<HTMLInputElement>(null);
  const composer = useRef<HTMLTextAreaElement>(null);

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

  // Opening the manager puts the cursor in "New note": type to add, or pick a note to edit.
  useEffect(() => {
    composer.current?.focus();
  }, [focusSignal]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        search.current?.focus();
        search.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    if (!undo) return;
    const id = setTimeout(() => setUndo(null), UNDO_MS);
    return () => clearTimeout(id);
  }, [undo]);

  const run = (p: Promise<unknown>) => p.catch((e) => setError(errorMessage(e)));

  const remove = (note: Note) => {
    void run(api.notesDelete(note.id).then(() => setUndo(note)));
  };

  return (
    <div className="notes">
      <div className="toolbar">
        <input
          ref={search}
          className="search"
          placeholder="Search notes… (accents optional)"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Escape" && setQuery("")}
        />
      </div>
      <Composer ref={composer} onError={setError} />
      {error && <div className="banner error">{error}</div>}
      {notes.length === 0 ? (
        <div className="empty">{query ? "No matching notes." : "No notes yet. Press the quick-note hotkey to add one."}</div>
      ) : (
        <ul className="note-list">
          {notes.map((n) => (
            <NoteItem
              key={n.id}
              note={n}
              onSave={(body) => run(api.notesUpdate(n.id, { body }))}
              onPin={() => run(api.notesUpdate(n.id, { pinned: !n.pinned }))}
              onDelete={() => remove(n)}
            />
          ))}
        </ul>
      )}
      {undo && (
        <div className="toast">
          Note deleted
          <button
            onClick={() => {
              void run(api.notesRestore(undo.id));
              setUndo(null);
            }}
          >
            Undo
          </button>
        </div>
      )}
    </div>
  );
}

const Composer = forwardRef<HTMLTextAreaElement, { onError: (e: string) => void }>(function Composer({ onError }, ref) {
  const [text, setText] = useState("");
  const submit = () => {
    const body = text.trim();
    if (!body) return;
    api
      .notesCreate(body)
      .then(() => setText(""))
      .catch((e) => onError(errorMessage(e)));
  };
  return (
    <textarea
      ref={ref}
      className="composer"
      placeholder="New note: Enter to save, Shift+Enter for newline · Ctrl+F to search"
      rows={Math.min(8, Math.max(1, text.split("\n").length))}
      value={text}
      onChange={(e) => setText(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          submit();
        }
      }}
    />
  );
});

function NoteItem({
  note,
  onSave,
  onPin,
  onDelete,
}: {
  note: Note;
  onSave: (body: string) => Promise<unknown>;
  onPin: () => void;
  onDelete: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(note.body);
  const area = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!editing) setDraft(note.body);
  }, [note.body, editing]);

  useEffect(() => {
    if (editing) {
      const el = area.current;
      el?.focus();
      el?.setSelectionRange(el.value.length, el.value.length);
    }
  }, [editing]);

  const commit = () => {
    setEditing(false);
    if (draft.trim() && draft.trim() !== note.body) void onSave(draft);
    else setDraft(note.body);
  };

  return (
    <li className={`note ${note.pinned ? "pinned" : ""}`}>
      <button className={`icon pin ${note.pinned ? "on" : ""}`} title={note.pinned ? "Unpin" : "Pin"} onClick={onPin}>
        {note.pinned ? "★" : "☆"}
      </button>
      <div className="note-main">
        {editing ? (
          <textarea
            ref={area}
            className="note-edit"
            value={draft}
            rows={Math.min(12, Math.max(2, draft.split("\n").length))}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commit}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) commit();
              if (e.key === "Escape") {
                e.stopPropagation();
                setDraft(note.body);
                setEditing(false);
              }
            }}
          />
        ) : (
          <div className="note-body" onClick={() => setEditing(true)} title="Click to edit">
            {note.body}
          </div>
        )}
        <div className="note-meta">
          {note.conflictOf && <span className="badge warn" title="Edited on two devices at once; this is the other version">⚠ conflict</span>}
          <span title={absoluteTime(note.updatedAt)}>{relativeTime(note.updatedAt)}</span>
        </div>
      </div>
      <button className="icon delete" title="Delete" onClick={onDelete}>
        ✕
      </button>
    </li>
  );
}
