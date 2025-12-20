pub const DEFAULT_TUI_CSS: &str = r#"
html, body {
  margin: 0;
  padding: 0;
  font-family: monospace;
  background: transparent;
  color: white;
}
h1, h2, h3, h4, h5, h6, p, ul, ol, li {
  margin: 0;
  padding: 0;
}
a { color: cyan; text-decoration: underline; }
strong, b { font-weight: bold; }
em, i { font-style: italic; }
code, pre { font-family: monospace; background: #111; padding: 0; }
img { display: block; }
ul, ol { margin: 0; padding-left: 0; }
button, input, select, textarea { background: #111; color: white; border: none; padding: 0; }
button:focus, input:focus, select:focus, textarea:focus { outline: none; background: #002b36; color: cyan; }
"#;
