import Image from "@tiptap/extension-image";
import { TaskItem, TaskList } from "@tiptap/extension-list";
import { Placeholder } from "@tiptap/extensions";
import { Markdown } from "@tiptap/markdown";
import { Editor, EditorContent, useEditor, useEditorState } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { useEffect, useRef } from "react";

import { Key, t } from "../../shared/i18n";
import { Icon, IconName } from "../../shared/Icon";
import { api, useBackendEvent } from "../../shared/ipc";

const SCHEME = "qdimg:";

/** Pictures already fetched, so scrolling or re-opening a note does not ask the backend again. Misses are not kept. */
const loaded = new Map<string, string>();

/** Pictures on screen that are not on this computer yet; asked for again when sync brings pictures. */
const waiting = new Set<HTMLImageElement>();

function showPicture(img: HTMLImageElement, src: string | null) {
  const hash = src?.startsWith(SCHEME) ? src.slice(SCHEME.length) : null;
  const cached = hash ? loaded.get(hash) : undefined;
  if (cached) {
    img.src = cached;
    img.classList.remove("missing");
    return;
  }
  img.removeAttribute("src");
  img.alt = t("notes.imageMissing");
  img.classList.add("missing");
  if (!hash) return;
  waiting.add(img);
  api
    .noteImage(hash)
    .then((url) => {
      if (!url) return;
      loaded.set(hash, url);
      waiting.delete(img);
      img.src = url;
      img.alt = "";
      img.classList.remove("missing");
    })
    .catch(() => {});
}

function retryWaiting() {
  for (const img of waiting) {
    if (!img.isConnected) waiting.delete(img);
    else showPicture(img, img.dataset.src ?? null);
  }
}

/** A picture in a note: `![](qdimg:<hash>)` in the Markdown, the real image on screen. */
const NotePicture = Image.extend({
  addNodeView() {
    return ({ node }) => {
      const dom = document.createElement("img");
      dom.draggable = false;
      dom.dataset.src = node.attrs.src;
      showPicture(dom, node.attrs.src);
      return {
        dom,
        update: (next) => {
          if (next.type !== node.type) return false;
          if (next.attrs.src !== node.attrs.src) {
            dom.dataset.src = next.attrs.src;
            showPicture(dom, next.attrs.src);
          }
          node = next;
          return true;
        },
      };
    };
  },
});

function imageFiles(data: DataTransfer | null): File[] {
  if (!data) return [];
  const files = Array.from(data.files).filter((f) => f.type.startsWith("image/"));
  if (files.length) return files;
  // Some webviews only list a pasted picture under `items`.
  return Array.from(data.items ?? [])
    .filter((i) => i.kind === "file" && i.type.startsWith("image/"))
    .map((i) => i.getAsFile())
    .filter((f): f is File => !!f);
}

function toBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result).split(",")[1] ?? "");
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

/**
 * Rich-text note body, stored as Markdown so search, sync and the plain
 * quick-note popup keep working. `onChange` fires only on user edits.
 */
export function NoteEditor({
  markdown,
  onChange,
  onReady,
  onError,
  autofocus,
}: {
  markdown: string;
  onChange: (markdown: string) => void;
  onReady?: (editor: Editor) => void;
  /** A picture could not be added. */
  onError?: (message: string) => void;
  autofocus: boolean;
}) {
  // Sync brought pictures: show the ones that were missing.
  useBackendEvent("notes://pictures", retryWaiting);

  const editorRef = useRef<Editor | null>(null);
  const onErrorRef = useRef(onError);
  onErrorRef.current = onError;

  /** Puts each picture into the note (at `pos`, else at the cursor). `store` shrinks and stores it in the backend and gives its id. */
  const addPictures = async (stores: (() => Promise<string | null>)[], pos?: number) => {
    for (const store of stores) {
      try {
        const hash = await store();
        const e = editorRef.current;
        if (!hash || !e) continue;
        const content = { type: "image", attrs: { src: SCHEME + hash } };
        const chain = e.chain().focus();
        (pos === undefined ? chain.insertContent(content) : chain.insertContentAt(pos, content)).run();
        pos = undefined;
      } catch {
        onErrorRef.current?.(t("notes.imageFailed"));
      }
    }
  };
  const addFiles = (files: File[], pos?: number) =>
    addPictures(
      files.map((file) => async () => api.noteImageAdd(await toBase64(file))),
      pos,
    );

  const editor = useEditor({
    extensions: [
      StarterKit.configure({ heading: { levels: [1, 2, 3] }, link: false }),
      TaskList,
      TaskItem.configure({ nested: true }),
      Placeholder.configure({ placeholder: t("notes.bodyPlaceholder") }),
      NotePicture,
      Markdown,
    ],
    content: markdown,
    contentType: "markdown",
    autofocus: autofocus ? "end" : false,
    editorProps: {
      attributes: { class: "note-rich", spellcheck: "false" },
      handlePaste: (_view, event) => {
        const data = event.clipboardData;
        const files = imageFiles(data);
        // Copying cells or formatted text brings a picture of it too: keep the text then.
        if (data?.getData("text/plain").trim()) return false;
        if (files.length) {
          void addFiles(files);
          return true;
        }
        // A screenshot that the webview does not show the page (WebKitGTK): the page sees an empty clipboard, so ask the backend.
        if (data && !data.getData("text/html") && !data.types.length) {
          void addPictures([() => api.noteImagePaste()]);
          return true;
        }
        return false;
      },
      handleDrop: (view, event) => {
        const files = imageFiles(event.dataTransfer);
        if (!files.length) return false;
        const at = view.posAtCoords({ left: event.clientX, top: event.clientY })?.pos;
        void addFiles(files, at);
        return true;
      },
    },
    onUpdate: ({ editor }) => onChange(editor.getMarkdown()),
  });

  useEffect(() => {
    editorRef.current = editor;
    if (editor && onReady) onReady(editor);
  }, [editor, onReady]);

  if (!editor) return null;
  return (
    <div className="note-editor">
      <Toolbar editor={editor} onPictures={(files) => void addFiles(files)} />
      <EditorContent editor={editor} className="note-rich-wrap" />
    </div>
  );
}

type Tool = {
  key: Key;
  label: string;
  /** Drawn instead of `label` where a letter is not the usual symbol. */
  icon?: IconName;
  shortcut: string;
  active: (e: Editor) => boolean;
  run: (e: Editor) => void;
};

const TOOLS: (Tool | "|")[] = [
  { key: "editor.bold", label: "B", shortcut: "Ctrl+B", active: (e) => e.isActive("bold"), run: (e) => e.chain().focus().toggleBold().run() },
  { key: "editor.italic", label: "I", shortcut: "Ctrl+I", active: (e) => e.isActive("italic"), run: (e) => e.chain().focus().toggleItalic().run() },
  { key: "editor.strike", label: "S", shortcut: "Ctrl+Shift+S", active: (e) => e.isActive("strike"), run: (e) => e.chain().focus().toggleStrike().run() },
  { key: "editor.code", label: "", icon: "code", shortcut: "Ctrl+E", active: (e) => e.isActive("code"), run: (e) => e.chain().focus().toggleCode().run() },
  "|",
  { key: "editor.heading", label: "H", shortcut: "Ctrl+Alt+2", active: (e) => e.isActive("heading"), run: (e) => e.chain().focus().toggleHeading({ level: 2 }).run() },
  { key: "editor.bullets", label: "", icon: "listBullet", shortcut: "Ctrl+Shift+8", active: (e) => e.isActive("bulletList"), run: (e) => e.chain().focus().toggleBulletList().run() },
  { key: "editor.numbers", label: "", icon: "listNumber", shortcut: "Ctrl+Shift+7", active: (e) => e.isActive("orderedList"), run: (e) => e.chain().focus().toggleOrderedList().run() },
  { key: "editor.tasks", label: "", icon: "checklist", shortcut: "Ctrl+Shift+9", active: (e) => e.isActive("taskList"), run: (e) => e.chain().focus().toggleTaskList().run() },
  "|",
  { key: "editor.quote", label: "", icon: "quote", shortcut: "Ctrl+Shift+B", active: (e) => e.isActive("blockquote"), run: (e) => e.chain().focus().toggleBlockquote().run() },
  { key: "editor.codeBlock", label: "", icon: "codeBlock", shortcut: "Ctrl+Alt+C", active: (e) => e.isActive("codeBlock"), run: (e) => e.chain().focus().toggleCodeBlock().run() },
];

function Toolbar({ editor, onPictures }: { editor: Editor; onPictures: (files: File[]) => void }) {
  const picker = useRef<HTMLInputElement>(null);
  // Re-render on selection changes so active states stay correct.
  const active = useEditorState({
    editor,
    selector: ({ editor: e }) => TOOLS.map((tool) => (tool === "|" ? false : tool.active(e))),
  });
  return (
    <div className="note-toolbar" role="toolbar">
      {TOOLS.map((tool, i) =>
        tool === "|" ? (
          <span key={i} className="tool-sep" />
        ) : (
          <button
            key={tool.key}
            type="button"
            className={`tool ${tool.key.replace("editor.", "")} ${active[i] ? "on" : ""}`}
            title={`${t(tool.key)} (${tool.shortcut})`}
            aria-pressed={active[i]}
            // Keep the selection in the editor when clicking a tool.
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => tool.run(editor)}
          >
            {tool.icon ? <Icon name={tool.icon} size={17} /> : tool.label}
          </button>
        ),
      )}
      <span className="tool-sep" />
      <button
        type="button"
        className="tool image"
        title={t("editor.image")}
        onMouseDown={(e) => e.preventDefault()}
        onClick={() => picker.current?.click()}
      >
        <Icon name="photo" size={17} />
      </button>
      <input
        ref={picker}
        type="file"
        accept="image/png,image/jpeg,image/gif,image/webp"
        multiple
        hidden
        onChange={(e) => {
          onPictures(Array.from(e.target.files ?? []));
          e.target.value = "";
        }}
      />
    </div>
  );
}
