use clap::{Parser, ValueEnum};

#[path = "catalog/mod.rs"]
mod catalog;

fn main() {
    let args = Args::parse();
    if args.list {
        for spec in catalog::apps() {
            println!("{}", spec.name);
        }
        return;
    }

    let name = args.component.unwrap_or_else(|| "dashboard".to_string());

    let Some(spec) = catalog::app_by_name(&name) else {
        eprintln!("unknown component: {name}");
        eprintln!("available:");
        for s in catalog::apps() {
            eprintln!("  {}", s.name);
        }
        std::process::exit(2);
    };

    let mut cfg = spec.cfg;
    if let Some(mode) = args.rendering_mode {
        cfg = cfg.with_rendering_mode(mode.into());
    }

    dioxus_tui::launch_cfg(spec.app, cfg).unwrap();
}

#[derive(Debug, Parser)]
#[command(about = "Launch an interactive example component")]
struct Args {
    /// Print available components and exit
    #[arg(long)]
    list: bool,

    /// Override rendering mode (useful for debugging)
    #[arg(long)]
    rendering_mode: Option<RenderingModeOpt>,

    /// Component name (default: dashboard)
    component: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum RenderingModeOpt {
    Visual,
    Debug,
    #[cfg(feature = "blitz")]
    BlitzTerminal,
    #[cfg(feature = "blitz")]
    BlitzGui,
    Headless,
}

impl From<RenderingModeOpt> for dioxus_tui::RenderingMode {
    fn from(value: RenderingModeOpt) -> Self {
        match value {
            RenderingModeOpt::Visual => dioxus_tui::RenderingMode::Visual,
            RenderingModeOpt::Debug => dioxus_tui::RenderingMode::Debug,
            #[cfg(feature = "blitz")]
            RenderingModeOpt::BlitzTerminal => dioxus_tui::RenderingMode::BlitzTerminal,
            #[cfg(feature = "blitz")]
            RenderingModeOpt::BlitzGui => dioxus_tui::RenderingMode::BlitzGui,
            RenderingModeOpt::Headless => dioxus_tui::RenderingMode::Headless,
        }
    }
}
