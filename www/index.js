const rust = import("../pkg");
import * as editor from "./editor.js";
import "./main.css";
import "./attribution.txt";

window.dbg = {};

var canvas = document.getElementById("glcanvas");
var username_div = document.getElementById("username");
var editor_div = document.getElementById("editor");
var scenario_select = document.getElementById("scenario");
var overlay = document.getElementById("overlay");
var doc_overlay = document.getElementById("doc-overlay");
var splash_overlay = document.getElementById("splash-overlay");
var mission_complete_overlay = document.getElementById(
  "mission-complete-overlay"
);
var doc_link = document.getElementById("doc_link");

function random_seed() {
  return Math.floor(Math.random() * 1e9);
}

var rust_module = null;
var game = null;
function initialize(m) {
  rust_module = m;
  window.dbg.rust = m;
  game = rust_module.create_game();
  window.dbg.game = game;

  editor.initialize(editor_div, {
    onExecute: (code) => {
      let scenario_name = scenario_select.value;
      game.save_code(scenario_name, code);
      game.start(scenario_name, random_seed(), code);
      window.setTimeout(() => canvas.focus(), 0);
    },
    getInitialCode: () => {
      let scenario_name = scenario_select.value;
      return game.get_initial_code(scenario_name);
    },
    getSolutionCode: () => {
      let scenario_name = scenario_select.value;
      return game.get_solution_code(scenario_name);
    },
  });

  initialize_scenario_list(game.get_scenarios());
  game.start("welcome", random_seed(), "");
  window.setTimeout(() => canvas.focus(), 0);

  username_div.textContent = game.get_username(game.get_userid());

  canvas.addEventListener("keydown", (e) => game.on_key_event(e));
  canvas.addEventListener("keyup", (e) => game.on_key_event(e));
  canvas.addEventListener("wheel", (e) => game.on_wheel_event(e));

  function render() {
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
    game.render();
    window.requestAnimationFrame(render);
  }
  render();
  canvas.style.visibility = "visible";
}

rust.then((m) => initialize(m)).catch(console.error);

var worker = new Worker(new URL("./worker.js", import.meta.url));
worker.onmessage = function (e) {
  game.on_snapshot(e.data.snapshot);
};

window.send_worker_request = function (msg) {
  worker.postMessage(msg);
};

function run_background_simulation(scenario_name, seed, code) {
  return new Promise((resolve, _) => {
    var worker = new Worker(new URL("./worker.js", import.meta.url));
    worker.postMessage({
      type: "RunScenario",
      scenario_name: scenario_name,
      seed: seed,
      code: code,
      nonce: 0,
    });
    worker.onmessage = function (e) {
      resolve(e.data.snapshot);
      worker.terminate();
    };
  });
}

window.start_background_simulations = function (scenario_name, code, n) {
  let promises = [];
  for (let i = 0; i < n; i++) {
    promises.push(run_background_simulation(scenario_name, i, code));
  }
  Promise.all(promises).then((r) => game.finished_background_simulations(r));
};

window.start_scenario = function (name) {
  scenario_select.value = name;
  editor.setText(game.get_saved_code(name));
  game.start(name, random_seed(), "");
  hide_overlay();
  window.setTimeout(() => canvas.focus(), 0);
};

function initialize_scenario_list(scenarios) {
  scenarios.forEach((scenario) => {
    var option = document.createElement("option");
    option.value = scenario;
    option.innerHTML = scenario;
    scenario_select.appendChild(option);
  });
  scenario_select.onchange = function (e) {
    start_scenario(e.target.value);
  };
}

window.send_telemetry = function (data) {
  if (document.location.hostname == "localhost") {
    return;
  }
  const xhr = new XMLHttpRequest();
  xhr.open("POST", "https://us-central1-oort-319301.cloudfunctions.net/upload");
  xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8");
  xhr.send(data);
  console.log("Sent telemetry: " + data);
};

function show_overlay(div) {
  div.onclick = (e) => e.stopPropagation();
  div.style.visibility = "visible";
  overlay.style.visibility = "visible";
  document.onkeydown = (e) => {
    if (e.key == "Escape") {
      hide_overlay();
    }
  };
}

function hide_overlay() {
  overlay.style.visibility = "hidden";
  doc_overlay.style.visibility = "hidden";
  splash_overlay.style.visibility = "hidden";
  mission_complete_overlay.style.visibility = "hidden";
  document.onkeydown = null;
}

overlay.onclick = hide_overlay;

doc_link.onclick = (_) => show_overlay(doc_overlay);

window.display_splash = function (contents) {
  splash_overlay.innerHTML = contents;
  show_overlay(splash_overlay);
};

window.display_mission_complete_overlay = function (
  scenario_name,
  time,
  code_size,
  next_scenario
) {
  document.getElementById("mission-complete-time").textContent =
    time.toPrecision(2);
  document.getElementById("mission-complete-code-size").textContent = code_size;
  document.getElementById("mission-complete-win-ratio").textContent =
    "Simulations in progress...";
  if (next_scenario) {
    document.getElementById("mission-complete-next").style.display = "inline";
    document.getElementById("mission-complete-next").onclick = () =>
      start_scenario(next_scenario);
    document.getElementById("mission-complete-no-next").style.display = "none";
  } else {
    document.getElementById("mission-complete-next").style.display = "none";
    document.getElementById("mission-complete-no-next").style.display =
      "inline";
  }
  document.getElementById("time-leaderboard").style.visibility = "hidden";
  document.getElementById("code-size-leaderboard").style.visibility = "hidden";
  show_overlay(mission_complete_overlay);

  const xhr = new XMLHttpRequest();
  xhr.open(
    "GET",
    "https://us-central1-oort-319301.cloudfunctions.net/leaderboard?scenario_name=" +
      scenario_name
  );
  xhr.onreadystatechange = function () {
    if (xhr.readyState === XMLHttpRequest.DONE && xhr.status == 200) {
      let data = JSON.parse(xhr.responseText);

      let update_leaderboard = function (tbody, rows, colname) {
        tbody.innerHTML = "";
        for (let row of rows) {
          var tr = document.createElement("tr");
          let add_td = function (content) {
            var td = document.createElement("td");
            td.textContent = content;
            tr.appendChild(td);
          };
          add_td(game.get_username(row.userid));
          add_td(row[colname]);
          tbody.appendChild(tr);
        }
      };

      update_leaderboard(
        document.getElementById("time-leaderboard-tbody"),
        data.lowest_time,
        "time"
      );
      update_leaderboard(
        document.getElementById("code-size-leaderboard-tbody"),
        data.lowest_code_size,
        "code_size"
      );
      document.getElementById("time-leaderboard").style.visibility = "unset";
      document.getElementById("code-size-leaderboard").style.visibility =
        "unset";
    }
  };
  xhr.send();
};

window.display_background_simulation_results = function (wins, total) {
  document.getElementById(
    "mission-complete-win-ratio"
  ).textContent = `Win ratio: ${wins}/${total}`;
};

window.display_errors = editor.displayErrors;
