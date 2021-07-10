const rust = import("../pkg");

function initialize(m) {
  var canvas = document.getElementById("glcanvas");

  function render() {
    canvas.width = canvas.clientWidth
    canvas.height = canvas.clientHeight
    m.render();
    window.requestAnimationFrame(render);
  }
  render();
}

rust.then((m) => initialize(m)).catch(console.error);
