const rust = import("../pkg");
import * as monaco from 'monaco-editor'

window.dbg = {};

var rust_module = null
function initialize(m) {
  var canvas = document.getElementById("glcanvas");
  rust_module = m;
  window.dbg.rust = m;

  m.initialize();
  m.start("asteroid");

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
  language: 'lua',
  theme: 'vs-dark',
  automaticLayout: true
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
