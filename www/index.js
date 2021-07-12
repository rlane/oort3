const rust = import("../pkg");
import * as monaco from 'monaco-editor'

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

var editor = monaco.editor.create(document.getElementById('editor'), {
  value: `log(42)`,
  language: 'lua',
  theme: 'vs-dark',
  automaticLayout: true
});
