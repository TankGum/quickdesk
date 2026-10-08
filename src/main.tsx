import React from "react";
import ReactDOM from "react-dom/client";

import { currentWindow } from "./shared/ipc";
import { ClipPopup } from "./windows/ClipPopup";
import { Main } from "./windows/Main";
import { NotePopup } from "./windows/NotePopup";
import "./styles.css";

// One bundle for every window; the Tauri window label picks the root view.
const views: Record<string, React.FC> = {
  main: Main,
  "note-popup": NotePopup,
  "clip-popup": ClipPopup,
};
const View = views[currentWindow.label] ?? Main;
document.documentElement.dataset.window = currentWindow.label;

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <View />
  </React.StrictMode>,
);
