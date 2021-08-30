const rust_import = import("../pkg");

let worker_promise = rust_import.then((x) => x.create_worker());

onmessage = async function (e) {
  let worker = await worker_promise;
  let response = worker.on_message(e.data);
  let transfer = [];
  if ("snapshot" in response) {
    transfer.push(response.snapshot.buffer);
  } else {
    console.warn("No snapshot in response");
  }
  postMessage(response, transfer);
};
