import { useRef, useState } from "react";

import { hideWindow, useEscapeToHide, useShown } from "../shared/ipc";

export function NotePopup() {
  const input = useRef<HTMLTextAreaElement>(null);
  const [text, setText] = useState("");
  const [lastSaved, setLastSaved] = useState<string | null>(null);

  useShown(() => setLastSaved(null), () => input.current);
  useEscapeToHide();

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      const body = text.trim();
      if (!body) return;
      // M2 persists via notes_create; M1 only exercises the capture flow.
      setLastSaved(body);
      setText("");
      void hideWindow();
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
      <div className="popup-hint">
        {lastSaved ? `Saved: ${lastSaved}` : "Enter ↵ save · Shift+Enter newline · Esc close"}
      </div>
    </div>
  );
}
