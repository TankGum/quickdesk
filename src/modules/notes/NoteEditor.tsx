import { TaskItem, TaskList } from "@tiptap/extension-list";
import { Placeholder } from "@tiptap/extensions";
import { Markdown } from "@tiptap/markdown";
import { Editor, EditorContent, useEditor, useEditorState } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { useEffect } from "react";

import { Key, t } from "../../shared/i18n";

/**
 * Rich-text note body, stored as Markdown so search, sync and the plain
 * quick-note popup keep working. `onChange` fires only on user edits.
 */
export function NoteEditor({
  markdown,
  onChange,
  onReady,
  autofocus,
}: {
  markdown: string;
  onChange: (markdown: string) => void;
  onReady?: (editor: Editor) => void;
  autofocus: boolean;
}) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({ heading: { levels: [1, 2, 3] }, link: false }),
      TaskList,
      TaskItem.configure({ nested: true }),
      Placeholder.configure({ placeholder: t("notes.bodyPlaceholder") }),
      Markdown,
    ],
    content: markdown,
    contentType: "markdown",
    autofocus: autofocus ? "end" : false,
    editorProps: { attributes: { class: "note-rich", spellcheck: "false" } },
    onUpdate: ({ editor }) => onChange(editor.getMarkdown()),
  });

  useEffect(() => {
    if (editor && onReady) onReady(editor);
  }, [editor, onReady]);

  if (!editor) return null;
  return (
    <div className="note-editor">
      <Toolbar editor={editor} />
      <EditorContent editor={editor} className="note-rich-wrap" />
    </div>
  );
}

type Tool = {
  key: Key;
  label: string;
  shortcut: string;
  active: (e: Editor) => boolean;
  run: (e: Editor) => void;
};

const TOOLS: (Tool | "|")[] = [
  { key: "editor.bold", label: "B", shortcut: "Ctrl+B", active: (e) => e.isActive("bold"), run: (e) => e.chain().focus().toggleBold().run() },
  { key: "editor.italic", label: "I", shortcut: "Ctrl+I", active: (e) => e.isActive("italic"), run: (e) => e.chain().focus().toggleItalic().run() },
  { key: "editor.strike", label: "S", shortcut: "Ctrl+Shift+S", active: (e) => e.isActive("strike"), run: (e) => e.chain().focus().toggleStrike().run() },
  { key: "editor.code", label: "</>", shortcut: "Ctrl+E", active: (e) => e.isActive("code"), run: (e) => e.chain().focus().toggleCode().run() },
  "|",
  { key: "editor.heading", label: "H", shortcut: "Ctrl+Alt+2", active: (e) => e.isActive("heading"), run: (e) => e.chain().focus().toggleHeading({ level: 2 }).run() },
  { key: "editor.bullets", label: "•", shortcut: "Ctrl+Shift+8", active: (e) => e.isActive("bulletList"), run: (e) => e.chain().focus().toggleBulletList().run() },
  { key: "editor.numbers", label: "1.", shortcut: "Ctrl+Shift+7", active: (e) => e.isActive("orderedList"), run: (e) => e.chain().focus().toggleOrderedList().run() },
  { key: "editor.tasks", label: "☑", shortcut: "Ctrl+Shift+9", active: (e) => e.isActive("taskList"), run: (e) => e.chain().focus().toggleTaskList().run() },
  "|",
  { key: "editor.quote", label: "❝", shortcut: "Ctrl+Shift+B", active: (e) => e.isActive("blockquote"), run: (e) => e.chain().focus().toggleBlockquote().run() },
  { key: "editor.codeBlock", label: "{ }", shortcut: "Ctrl+Alt+C", active: (e) => e.isActive("codeBlock"), run: (e) => e.chain().focus().toggleCodeBlock().run() },
];

function Toolbar({ editor }: { editor: Editor }) {
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
            {tool.label}
          </button>
        ),
      )}
    </div>
  );
}
