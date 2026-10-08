import { useRef } from "react";

import { useEscapeToHide, useShown } from "../shared/ipc";

export function ClipPopup() {
  const input = useRef<HTMLInputElement>(null);

  useShown(() => {}, () => input.current);
  useEscapeToHide();

  return (
    <div className="popup clip">
      <input ref={input} className="popup-search" placeholder="🔍 Search clipboard..." autoFocus />
      <div className="empty">Clipboard history arrives in Milestone 4.</div>
    </div>
  );
}
