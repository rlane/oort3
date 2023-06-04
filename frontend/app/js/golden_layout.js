var goldenLayout;
let configVersion = 4;

function make_config() {
  let editor_width_pct = 100.0 - 61.8;
  let window_width = document.documentElement.clientWidth;
  let min_editor_width =
    68 + 14 + document.getElementById("text-size").clientWidth;
  if ((window_width * editor_width_pct) / 100.0 < min_editor_width) {
    editor_width_pct = (100.0 * min_editor_width) / window_width;
  }
  if (editor_width_pct > 50.0) {
    editor_width_pct = 50.0;
  }

  var config = {
    settings: {
      showPopoutIcon: false,
      showCloseIcon: false,
      selectionEnabled: true,
    },
    content: [
      {
        type: "row",
        content: [
          {
            type: "stack",
            width: editor_width_pct,
            content: [
              {
                type: "component",
                title: "Editor (Player)",
                componentName: "Editor",
                componentState: { team: 0 },
                isClosable: false,
                id: "editor.player",
              },
              {
                type: "component",
                componentName: "Editor",
                title: "Editor (Opponent)",
                componentState: { team: 1 },
                isClosable: false,
                id: "editor.opponent",
              },
              {
                type: "component",
                componentName: "Versions",
                componentState: {},
                isClosable: false,
                id: "versions",
              },
            ],
          },
          {
            type: "stack",
            content: [
              {
                type: "component",
                componentName: "Simulation",
                componentState: {},
                isClosable: false,
                id: "simulation",
              },
              {
                type: "component",
                componentName: "Quick Reference",
                componentState: {},
                isClosable: false,
                id: "quick_reference",
              },
              {
                type: "component",
                componentName: "Compiler Output",
                componentState: {},
                isClosable: false,
                id: "compiler_output",
              },
              {
                type: "component",
                componentName: "Leaderboard",
                componentState: {},
                isClosable: false,
                id: "leaderboard",
              },
            ],
          },
        ],
      },
    ],
  };

  return config;
}

function load_config() {
  let json = localStorage.getItem("layout." + configVersion);
  if (json == null) {
    return null;
  }
  return JSON.parse(json);
}

export function init() {
  var config = load_config();
  if (config == null) {
    config = make_config();
  }

  goldenLayout = new GoldenLayout(
    config,
    document.getElementById("goldenlayout")
  );
  goldenLayout.registerComponent(
    "Welcome",
    function (container, componentState) {
      container.getElement()[0].style.overflow = "auto";
      container
        .getElement()[0]
        .appendChild(document.getElementById("welcome-window"));
    }
  );
  goldenLayout.registerComponent(
    "Editor",
    function (container, componentState) {
      container.getElement()[0].id = "editor-window-" + componentState.team;
    }
  );
  goldenLayout.registerComponent(
    "Simulation",
    function (container, componentState) {
      container.getElement()[0].id = "simulation-window";
    }
  );
  goldenLayout.registerComponent(
    "Quick Reference",
    function (container, componentState) {
      container.getElement()[0].style.overflow = "auto";
      container.getElement()[0].id = "documentation-window";
    }
  );
  goldenLayout.registerComponent(
    "Compiler Output",
    function (container, componentState) {
      container.getElement()[0].style.overflow = "auto";
      container.getElement()[0].id = "compiler-output-window";
    }
  );
  goldenLayout.registerComponent(
    "Leaderboard",
    function (container, componentState) {
      container.getElement()[0].style.overflow = "auto";
      container.getElement()[0].id = "leaderboard-window";
    }
  );
  goldenLayout.registerComponent(
    "Versions",
    function (container, componentState) {
      container.getElement()[0].style.overflow = "auto";
      container.getElement()[0].id = "versions-window";
    }
  );
  goldenLayout.init();

  window.goldenLayout = goldenLayout;

  window.onbeforeunload = function () {
    localStorage.setItem(
      "layout." + configVersion,
      JSON.stringify(goldenLayout.toConfig())
    );
    return null;
  };
}

export function update_size() {
  goldenLayout.updateSize();
}

function welcome_component() {
  return {
    type: "component",
    title: "Welcome",
    componentName: "Welcome",
    componentState: {},
    isClosable: true,
    id: "welcome",
  };
}

export function show_welcome(visible) {
  let tabs = goldenLayout.root.contentItems[0].contentItems[0];
  let existing = tabs.getItemsById("welcome");
  let currently_visible = existing.length != 0;
  if (visible != currently_visible) {
    if (visible) {
      tabs.addChild(welcome_component(), 0);
    } else {
      document
        .getElementById("welcome-hidden")
        .appendChild(document.getElementById("welcome-window"));
      tabs.removeChild(existing[0], false);
    }
  }
}

export function select_tab(id) {
  let item = goldenLayout.root.getItemsById(id)[0];
  if (item == null) {
    console.log("Could not find item " + id);
    return;
  }
  var prev = null;
  while (item != null) {
    if (item.isStack) {
      item.setActiveContentItem(prev);
    }
    prev = item;
    item = item.parent;
  }
}
