pub const DEFAULT_TUI_CSS: &str = r#"
html, body {
  margin: 0;
  padding: 0;
  font-family: monospace;
  background: black;
  color: white;
}
a { color: cyan; text-decoration: underline; }
strong, b { font-weight: bold; }
em, i { font-style: italic; }
code, pre { font-family: monospace; background: #111; padding: 0; }
ul, ol { margin: 0; padding-left: 2ch; }
button, input, select, textarea { background: #111; color: white; border: none; padding: 0; }
button:focus, input:focus, select:focus, textarea:focus { outline: none; background: #002b36; color: cyan; }
"#;
