export function init() {
  console.log("Initializing completions");
  monaco.languages.registerCompletionItemProvider("rust", {
    provideCompletionItems: function (model, position) {
      return model.completer.complete(position);
    },
  });
}
