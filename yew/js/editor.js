/* TODO
  monaco.languages.registerCompletionItemProvider("rust", {
    provideCompletionItems: getCompletions,
  });
*/

var suggestion_terms = [
  // Ship
  "ship.position",
  "ship.velocity",
  "ship.heading",
  "ship.angular_velocity",
  "ship.accelerate",
  "ship.torque",
  "ship.fire_weapon",
  "ship.launch_missile",
  "ship.class",
  "ship.explode",

  // Radar
  "radar.set_heading",
  "radar.set_width",
  "radar.scan",

  // Scalar Math
  "abs",
  "sin",
  "sqrt",
  "log",
  "min",
  "PI()",
  "E()",

  // Vector Math
  "vec2",
  ".magnitude",
  ".normalize",
  ".rotate",
  ".angle",
  ".dot",
  ".distance",

  // Miscellaneous
  "print",
  "rng.next",
  "angle_diff",
  "dbg.line",
];

function getCompletions(model, position) {
  var word = model.getWordUntilPosition(position);
  var range = {
    startLineNumber: position.lineNumber,
    endLineNumber: position.lineNumber,
    startColumn: word.startColumn,
    endColumn: word.endColumn,
  };
  var suggestions = [];
  for (var term of suggestion_terms) {
    suggestions.push({
      label: term,
      kind: monaco.languages.CompletionItemKind.Function,
      insertText: term,
      range: range,
    });
  }
  return {
    suggestions: suggestions,
  };
}

let current_decorations = [];
export function display_errors(errors) {
  if (true) {
    return;
  }
  let new_decorations = [];
  for (let error of errors) {
    new_decorations.push({
      range: new monaco.Range(error.line, 1, error.line, 1),
      options: {
        isWholeLine: true,
        className: "errorDecoration",
        hoverMessage: { value: error.msg },
      },
    });
  }
  current_decorations = editor.deltaDecorations(
    current_decorations,
    new_decorations
  );
}
