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
  multiple: true,
};

export class FileHandle {
  constructor(handle) {
    this._handle = handle;
  }

  async read() {
    if ("getFile" in this._handle) {
      let file = await this._handle.getFile();
      return file.text();
    } else {
      let file = await new Promise((resolve) => this._handle.file(resolve));
      return await file.text();
    }
  }

  async name() {
    if ("getFile" in this._handle) {
      let file = await this._handle.getFile();
      return file.name;
    } else {
      let file = await new Promise((resolve) => this._handle.file(resolve));
      return file.name;
    }
  }
}

export async function open() {
  let handles = await window.showOpenFilePicker(pickerOpts);
  let results = [];
  for (let handle of handles) {
    results.push(new FileHandle(handle));
  }
  return results;
}

export class DirectoryHandle {
  constructor(handle) {
    this._handle = handle;
  }

  async getFiles() {
    let files = [];
    for await (let entry of this._handle.values()) {
      if (entry.kind === "file") {
        files.push(new FileHandle(entry));
      }
    }
    return files;
  }
}

export async function openDirectory() {
  return new DirectoryHandle(await window.showDirectoryPicker());
}
