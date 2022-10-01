export function send_telemetry(data) {
  if (document.location.hostname == "localhost") {
    console.log("Skipping sending telemetry: " + data);
    return;
  }
  const xhr = new XMLHttpRequest();
  xhr.open("POST", "https://telemetry.oort.rs/post");
  xhr.setRequestHeader("Content-Type", "application/json;charset=UTF-8");
  xhr.send(data);
  console.log("Sent telemetry: " + data);
}
