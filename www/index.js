const rust = import("../pkg");
import * as monaco from 'monaco-editor'
import "./main.css";

window.dbg = {};

var rust_module = null
function initialize(m) {
  var canvas = document.getElementById("glcanvas");
  rust_module = m;
  window.dbg.rust = m;

  m.initialize();
  m.start("welcome");

  canvas.addEventListener('keydown', m.on_key_event);
  canvas.addEventListener('keyup', m.on_key_event);

  function render() {
    canvas.width = canvas.clientWidth
    canvas.height = canvas.clientHeight
    m.render();
    window.requestAnimationFrame(render);
  }
  render();
}

rust.then((m) => initialize(m)).catch(console.error);

var editor = monaco.editor.create(document.getElementById('editor'), {
  value: `\
// Welcome to Oort.
// Select a scenario from the list in the top-right of the page.
// If you're new, start with "tutorial01".`,
  language: 'rust',
  theme: 'vs-dark',
  automaticLayout: true,
  largeFileOptimizations: false,
  minimap: { enabled: false },
});
window.dbg.editor = editor;

editor.addAction({
  id: 'oort-execute',
  label: 'Execute',
  keybindings: [
    monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter,
  ],
  precondition: null,
  keybindingContext: null,
  contextMenuGroupId: 'navigation',
  contextMenuOrder: 1.5,
  run: function(ed) {
    rust_module.start(document.getElementById('scenario').value);
    rust_module.upload_code(ed.getValue());
    return null;
  }
});

var scenario_select = document.getElementById('scenario');
var scenarios = ['welcome', 'tutorial01', 'tutorial02', 'tutorial03', 'asteroid'];
scenarios.forEach((scenario) => {
  var option = document.createElement('option');
  option.value = scenario;
  option.innerHTML = scenario;
  scenario_select.appendChild(option);
});
scenario_select.onchange = function(e) {
  rust_module.start(e.target.value);
  editor.setValue(rust_module.get_initial_code());
}
