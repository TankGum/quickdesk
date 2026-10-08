import React from "react";
import ReactDOM from "react-dom/client";

import { initI18n } from "./shared/i18n";
import { currentWindow } from "./shared/ipc";
import { ClipPopup } from "./windows/ClipPopup";
import { Main } from "./windows/Main";
import { NotePopup } from "./windows/NotePopup";
import { UsagePopup } from "./windows/UsagePopup";
import "./styles.css";

// One bundle for every window; the Tauri window label picks the root view.
const views: Record<string, React.FC> = {
  main: Main,
  "note-popup": NotePopup,
  "clip-popup": ClipPopup,
  "usage-popup": UsagePopup,
};
const View = views[currentWindow.label] ?? Main;
document.documentElement.dataset.window = currentWindow.label;

// Load the chosen language first so nothing flashes in the wrong one.
void initI18n().finally(() =>
  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <View />
    </React.StrictMode>,
  ),
);
