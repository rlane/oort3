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
            type: "component",
            componentName: "Editor",
            componentState: {},
            isClosable: false,
          },
          {
            type: "component",
            componentName: "Simulation",
            componentState: {},
            isClosable: false,
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
    "Editor",
    function (container, componentState) {
      container.getElement()[0].id = "editor-window";
    }
  );
  goldenLayout.registerComponent(
    "Simulation",
    function (container, componentState) {
      container.getElement()[0].id = "simulation-window";
    }
  );
  goldenLayout.init();

  window.goldenLayout = goldenLayout;
}

export function update_size() {
  goldenLayout.updateSize();
}
