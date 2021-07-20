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
  canvas.addEventListener('wheel', m.on_wheel_event);

  function render() {
    canvas.width = canvas.clientWidth
    canvas.height = canvas.clientHeight
    m.render();
    window.requestAnimationFrame(render);
  }
  render();
  canvas.style.visibility = 'visible';
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
var scenarios = ['welcome', 'tutorial01', 'tutorial02', 'tutorial03', 'tutorial04', 'asteroid'];
scenarios.forEach((scenario) => {
  var option = document.createElement('option');
  option.value = scenario;
  option.innerHTML = scenario;
  scenario_select.appendChild(option);
});
scenario_select.onchange = function(e) {
  rust_module.start(e.target.value);
  editor.setValue(rust_module.get_initial_code());
  splash_div.style.visibility = 'hidden';
  document.onkeydown = null;
}

var doc_link = document.getElementById('doc_link');
var doc_div = document.getElementById('doc');
doc_link.onclick = (e) => {
  doc_div.style.visibility = 'visible';
  document.onkeydown = (e) => {
    if (e.key == 'Escape') {
      doc_div.style.visibility = 'hidden';
      document.onkeydown = null;
    }
  }
}

window.send_telemetry = function(data) {
  const xhr = new XMLHttpRequest();
  xhr.open("POST", "https://us-central1-oort-319301.cloudfunctions.net/upload");
  xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8");
  xhr.send(data);
}

var splash_div = document.getElementById('splash');
window.display_splash = function(contents) {
  splash_div.innerHTML = contents;
  splash_div.style.visibility = 'visible';
  document.onkeydown = (e) => {
    if (e.key == 'Escape') {
      splash_div.style.visibility = 'hidden';
      document.onkeydown = null;
    }
  }
}
