import * as monaco from "https://cdn.jsdelivr.net/npm/monaco-editor@0.39.0/+esm";

window.editor = monaco.editor.create(document.querySelector("#monaco"), {
  value: ['return div { "hello dioscript!" };'].join("\n"),
  fontSize: 13,
});



