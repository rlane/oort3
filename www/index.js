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
print("Script started");
debug("Debug log");
fn tick() {
  api.thrust_main(1e5);
  api.thrust_lateral(1e5);
  api.thrust_angular(1e5);
  api.fire_weapon(0);
  //api.explode();
}`,
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
    rust_module.upload_code(ed.getValue());
    return null;
  }
});

window.set_editor_code = function(code) {
  console.log("set_editor_code");
  editor.setValue(code);
};

var scenario_select = document.getElementById('scenario');
var scenarios = ['welcome', 'tutorial01', 'tutorial02', 'asteroid'];
scenarios.forEach((scenario) => {
  var option = document.createElement('option');
  option.value = scenario;
  option.innerHTML = scenario;
  scenario_select.appendChild(option);
});
scenario_select.onchange = (e) => rust_module.start(e.target.value);
