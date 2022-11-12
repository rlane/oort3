var goldenLayout;

export function init() {
  var config = {
    settings: {
      showPopoutIcon: false,
      showCloseIcon: false,
    },
    content: [
      {
        type: "row",
        content: [
          {
            type: "stack",
            content: [
              {
                type: "component",
                title: "Editor (Player)",
                componentName: "Editor",
                componentState: { team: 0 },
                isClosable: false,
              },
              {
                type: "component",
                componentName: "Editor",
                title: "Editor (Opponent)",
                componentState: { team: 1 },
                isClosable: false,
              },
            ],
          },
          {
            type: "stack",
            width: 61.8,
            content: [
              {
                type: "component",
                componentName: "Simulation",
                componentState: {},
                isClosable: false,
              },
              {
                type: "component",
                componentName: "Quick Reference",
                componentState: {},
                isClosable: false,
              },
            ],
          },
        ],
      },
    ],
  };

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
  goldenLayout.init();

  window.goldenLayout = goldenLayout;
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
