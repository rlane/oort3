import "./style.css";
import "./js/editor.js";
import "./js/telemetry.js";

import("./pkg").then((module) => {
  module.run_app();
});
