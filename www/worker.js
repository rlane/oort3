const rust_import = import("../pkg");

(async function () {
  let rust = await rust_import;
  rust.worker_initialize();
})();

onmessage = async function (e) {
  let rust = await rust_import;
  if (e.data.type == "run") {
    postMessage(
      rust.worker_run_scenario(
        e.data.scenario_name,
        BigInt(e.data.seed),
        e.data.code
      )
    );
  } else if (e.data.type == "start") {
    postMessage(
      rust.worker_start_scenario(
        e.data.scenario_name,
        BigInt(e.data.seed),
        e.data.code
      )
    );
  } else if (e.data.type == "request_snapshot") {
    let snapshot = rust.worker_request_snapshot();
    postMessage(snapshot, [snapshot.buffer]);
  }
};
