import "./style.css";
import "./editor.js";

import("./pkg").then((module) => {
  module.run_app();
});
