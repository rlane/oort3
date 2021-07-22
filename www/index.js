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

window.start_scenario = function(name) {
  rust_module.start(name);
  editor.setValue(rust_module.get_initial_code());
  hide_overlay();
}

var scenario_select = document.getElementById('scenario');
var scenarios = ['welcome', 'tutorial01', 'tutorial02', 'tutorial03', 'tutorial04', 'tutorial05', 'asteroid'];
scenarios.forEach((scenario) => {
  var option = document.createElement('option');
  option.value = scenario;
  option.innerHTML = scenario;
  scenario_select.appendChild(option);
});
scenario_select.onchange = function(e) {
  start_scenario(e.target.value);
}

window.send_telemetry = function(data) {
  const xhr = new XMLHttpRequest();
  xhr.open("POST", "https://us-central1-oort-319301.cloudfunctions.net/upload");
  xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8");
  xhr.send(data);
  console.log("Sent telemetry: " + data);
}

var overlay = document.getElementById('overlay');
var doc_overlay = document.getElementById('doc-overlay');
var splash_overlay = document.getElementById('splash-overlay');

function show_overlay(div) {
  div.onclick = (e) => e.stopPropagation();
  div.style.visibility = 'visible';
  overlay.style.visibility = 'visible';
  document.onkeydown = (e) => {
    if (e.key == 'Escape') {
      hide_overlay();
    }
  }
}

function hide_overlay() {
  overlay.style.visibility = 'hidden'
  doc_overlay.style.visibility = 'hidden'
  splash_overlay.style.visibility = 'hidden'
  document.onkeydown = null;
}

overlay.onclick = hide_overlay;

var doc_link = document.getElementById('doc_link');
doc_link.onclick = (e) => show_overlay(doc_overlay);

window.display_splash = function(contents) {
  splash_overlay.innerHTML = contents;
  show_overlay(splash_overlay);
}
