const rust_import = import("../pkg");

let worker_promise = rust_import.then((x) => x.create_worker());

onmessage = async function (e) {
  let worker = await worker_promise;
  if (e.data.type == "start") {
    postMessage(
      worker.start_scenario(
        e.data.scenario_name,
        BigInt(e.data.seed),
        e.data.code
      )
    );
  } else if (e.data.type == "run") {
    postMessage(
      worker.run_scenario(
        e.data.scenario_name,
        BigInt(e.data.seed),
        e.data.code
      )
    );
  } else if (e.data.type == "request_snapshot") {
    let snapshot = worker.request_snapshot(e.data.nonce);
    postMessage(snapshot, [snapshot.buffer]);
  }
};
