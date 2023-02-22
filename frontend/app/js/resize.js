export function start(cb) {
  const resizeObserver = new ResizeObserver(cb);
  resizeObserver.observe(document.documentElement);
  resizeObserver.observe(document.getElementById("editor-window-0"));
  resizeObserver.observe(document.getElementById("editor-window-1"));
}
