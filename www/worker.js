console.log("Hello world from worker");
const rust = import("../pkg");
rust.then((m) => m.initialize_worker()).catch(console.error);

onmessage = function(e) {
  console.log('Worker: Message received from main script: ' + JSON.stringify(e.data));
  postMessage(e.data + "bar");
}
