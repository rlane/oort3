console.log("Hello world from worker");
const rust = import("../pkg");
rust.then((m) => m.initialize_worker()).catch(console.error);
