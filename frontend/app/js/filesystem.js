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

export class FileHandle {
  constructor(handle) {
    this._handle = handle;
  }

  async read() {
    let file = await this._handle.getFile();
    return file.text();
  }
}

export async function open() {
  let [handle] = await window.showOpenFilePicker(pickerOpts);
  if (handle.kind != "file") {
    throw "Not a file";
  }
  return new FileHandle(handle);
}
