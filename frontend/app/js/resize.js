export function start(cb) {
  const resizeObserver = new ResizeObserver(cb);
  resizeObserver.observe(document.documentElement);
}
