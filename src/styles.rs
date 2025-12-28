pub const DEFAULT_TUI_CSS: &str = r#"
html, body {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
  font-family: monospace;
  line-height: 1;
  background: transparent;
  color: white;
}

/* `main` is block-level in the web UA stylesheet. */
main {
  display: block;
}

/* Dioxus apps are typically mounted under `body > main`.
   Make that container fill the viewport so children using percent sizes resolve correctly. */
body > main {
  width: 100%;
  height: 100%;
}

/* Basic HTML display defaults (Blitz doesn't inject a full UA stylesheet). */
div, p, pre, ul, ol, li, header, footer, section, article, nav, table, thead, tbody, tfoot, tr, td, th,
textarea, span, a, strong, b, em, i, code, h1, h2, h3, h4, h5, h6 {
  display: block;
  line-height: 1;
}
span, a, strong, b, em, i, code {
  display: inline;
  line-height: 1;
}
h1, h2, h3, h4, h5, h6, p, ul, ol, li {
  margin: 0;
  padding: 0;
}
h1, h2, h3, h4, h5, h6 { font-size: 1em; }
a { color: cyan; text-decoration: underline; }
strong, b { font-weight: bold; }
em, i { font-style: italic; }
code, pre { font-family: monospace; background: #111; padding: 0; }
img { display: block; }
ul, ol { margin: 0; padding-left: 0; }
button, input, select, textarea { background: #111; color: white; border: none; padding: 0; }
button:focus, input:focus, select:focus, textarea:focus { outline: none; background: #002b36; color: cyan; }
"#;
