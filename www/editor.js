import * as monaco from "monaco-editor";

var editor = null;

export function initialize(editor_div, callbacks) {
  editor = monaco.editor.create(editor_div, {
    value: `\
  // Welcome to Oort.
  // Select a scenario from the list in the top-right of the page.
  // If you're new, start with 'tutorial01'.`,
    language: "rust",
    theme: "vs-dark",
    automaticLayout: true,
    largeFileOptimizations: false,
    minimap: { enabled: false },
  });
  window.dbg.editor = editor;

  editor.addAction({
    id: "oort-execute",
    label: "Execute",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
    contextMenuGroupId: "navigation",
    contextMenuOrder: 1.5,
    run: function (ed) {
      callbacks.onExecute(ed.getValue());
      return null;
    },
  });

  editor.addAction({
    id: "oort-restore-initial-code",
    label: "Restore initial code",
    contextMenuGroupId: "navigation",
    contextMenuOrder: 1.6,
    run: function (_) {
      editor.setValue(callbacks.getInitialCode());
      return null;
    },
  });

  editor.addAction({
    id: "oort-load-solution",
    label: "Load solution",
    run: function (_) {
      editor.setValue(callbacks.getSolutionCode());
      return null;
    },
  });
}

export function setText(text) {
  editor.setValue(text);
}

let current_decorations = [];
export function displayErrors(errors) {
  let new_decorations = [];
  for (let error of errors) {
    new_decorations.push({
      range: new monaco.Range(error.line, 1, error.line, 1),
      options: {
        isWholeLine: true,
        className: "errorDecoration",
        hoverMessage: { value: error.msg },
      },
    });
  }
  current_decorations = editor.deltaDecorations(
    current_decorations,
    new_decorations
  );
}
