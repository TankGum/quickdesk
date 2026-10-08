import { useRef, useState } from "react";

import { api, errorMessage, hideWindow, useEscapeToHide, useShown } from "../shared/ipc";

export function NotePopup() {
  const input = useRef<HTMLTextAreaElement>(null);
  // The draft survives Esc / losing focus; it is cleared only after a save.
  const [text, setText] = useState("");
  const [status, setStatus] = useState<{ kind: "saved" | "error"; text: string } | null>(null);
  const [saving, setSaving] = useState(false);

  useShown(() => setStatus(null), () => input.current);
  useEscapeToHide();

  const save = async (openMain: boolean) => {
    const body = text.trim();
    if (!body || saving) return;
    setSaving(true);
    try {
      await api.notesCreate(body);
      setText("");
      setStatus({ kind: "saved", text: "Saved" });
      if (openMain) await api.show("notes");
      else await hideWindow();
    } catch (e) {
      setStatus({ kind: "error", text: errorMessage(e) });
    } finally {
      setSaving(false);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void save(e.ctrlKey || e.metaKey);
    }
  };

  return (
    <div className="popup">
      <textarea
        ref={input}
        className="popup-input"
        placeholder="What do you want to remember?"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={onKeyDown}
        rows={3}
        autoFocus
      />
      <div className={`popup-hint ${status?.kind === "error" ? "error" : ""}`}>
        {status ? status.text : "Enter ↵ save · Shift+Enter newline · Ctrl+Enter save & open · Esc close"}
      </div>
    </div>
  );
}
