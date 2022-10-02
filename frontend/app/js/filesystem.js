const pickerOpts = {
  types: [
    {
      description: "Rust",
      accept: {
        "text/plain": [".rs"],
      },
    },
  ],
  excludeAcceptAllOption: true,
  multiple: false,
};

let fileHandle = null;

export async function load_file() {
  [fileHandle] = await window.showOpenFilePicker(pickerOpts);
  if (fileHandle.kind != "file") {
    throw "Not a file";
  }
  let file = await fileHandle.getFile();
  return file.text();
}

export async function reload_file() {
  if (fileHandle == null) {
    return await load_file();
  }
  let file = await fileHandle.getFile();
  return file.text();
}
