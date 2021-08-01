const rust = import("../pkg");
import * as monaco from 'monaco-editor'
import "./main.css";
import "./attribution.txt"

window.dbg = {};

var canvas = document.getElementById("glcanvas");

var rust_module = null
function initialize(m) {
  rust_module = m;
  window.dbg.rust = m;

  m.initialize();
  initialize_scenario_list(m.get_scenarios());
  rust_module.start("welcome", "");
  window.setTimeout(() => canvas.focus(), 0);

  document.getElementById("username").textContent = m.get_username(m.get_userid());

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
    rust_module.start(document.getElementById('scenario').value, ed.getValue());
    window.setTimeout(() => canvas.focus(), 0);
    return null;
  }
});

var scenario_select = document.getElementById('scenario');

window.start_scenario = function(name) {
  scenario_select.value = name;
  rust_module.start(name, "");
  editor.setValue(rust_module.get_initial_code());
  hide_overlay();
  window.setTimeout(() => canvas.focus(), 0);
}

function initialize_scenario_list(scenarios) {
  scenarios.forEach((scenario) => {
    var option = document.createElement('option');
    option.value = scenario;
    option.innerHTML = scenario;
    scenario_select.appendChild(option);
  });
  scenario_select.onchange = function(e) {
    start_scenario(e.target.value);
  }
}

window.send_telemetry = function(data) {
  if (document.location.hostname == 'localhost') {
    return;
  }
  const xhr = new XMLHttpRequest();
  xhr.open("POST", "https://us-central1-oort-319301.cloudfunctions.net/upload");
  xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8");
  xhr.send(data);
  console.log("Sent telemetry: " + data);
}

var overlay = document.getElementById('overlay');
var doc_overlay = document.getElementById('doc-overlay');
var splash_overlay = document.getElementById('splash-overlay');
var mission_complete_overlay = document.getElementById('mission-complete-overlay');

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
  mission_complete_overlay.style.visibility = 'hidden'
  document.onkeydown = null;
}

overlay.onclick = hide_overlay;

var doc_link = document.getElementById('doc_link');
doc_link.onclick = (e) => show_overlay(doc_overlay);

window.display_splash = function(contents) {
  splash_overlay.innerHTML = contents;
  show_overlay(splash_overlay);
}

window.display_mission_complete_overlay = function(scenario_name, time, code_size, next_scenario) {
  document.getElementById('mission-complete-time').textContent = time.toPrecision(2);
  document.getElementById('mission-complete-code-size').textContent = code_size;
  if (next_scenario) {
    document.getElementById('mission-complete-next').style.display = 'inline';
    document.getElementById('mission-complete-next').onclick = () => start_scenario(next_scenario);
    document.getElementById('mission-complete-no-next').style.display = 'none';
  } else {
    document.getElementById('mission-complete-next').style.display = 'none';
    document.getElementById('mission-complete-no-next').style.display = 'inline';
  }
  document.getElementById('time-leaderboard').style.visibility = 'hidden';
  document.getElementById('code-size-leaderboard').style.visibility = 'hidden';
  show_overlay(mission_complete_overlay);

  const xhr = new XMLHttpRequest();
  xhr.open("GET", "https://us-central1-oort-319301.cloudfunctions.net/leaderboard?scenario_name=" + scenario_name);
  xhr.onreadystatechange = function () {
    if(xhr.readyState === XMLHttpRequest.DONE && xhr.status == 200) {
      let data = JSON.parse(xhr.responseText);

      let update_leaderboard = function(tbody, rows, colname) {
        tbody.innerHTML = '';
        for (let row of rows) {
          var tr = document.createElement('tr');
          let add_td = function(content) {
            var td = document.createElement('td');
            td.textContent = content;
            tr.appendChild(td);
          }
          add_td(rust_module.get_username(row.userid));
          add_td(row[colname]);
          tbody.appendChild(tr);
        }
      };

      update_leaderboard(document.getElementById('time-leaderboard-tbody'), data.lowest_time, 'time');
      update_leaderboard(document.getElementById('code-size-leaderboard-tbody'), data.lowest_code_size, 'code_size');
      document.getElementById('time-leaderboard').style.visibility = 'unset';
      document.getElementById('code-size-leaderboard').style.visibility = 'unset';
    }
  }
  xhr.send();
}

let current_decorations = [];
window.display_errors = function(errors) {
  let new_decorations = [];
  for (let error of errors) {
    new_decorations.push({
      range: new monaco.Range(error.line,1,error.line,1),
      options: {
        isWholeLine: true,
        className: 'errorDecoration',
        hoverMessage: { value: error.msg },
      }
    });
  }
  current_decorations = editor.deltaDecorations(current_decorations, new_decorations);
};
