use dioxus_tui::RenderRequest;

#[path = "catalog/mod.rs"]
mod catalog;

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| "dashboard".to_string());
    if name == "--list" {
        for spec in catalog::apps() {
            println!("{}", spec.name);
        }
        return;
    }

    let Some(spec) = catalog::app_by_name(&name) else {
        eprintln!("unknown component: {name}");
        eprintln!("available:");
        for s in catalog::apps() {
            eprintln!("  {}", s.name);
        }
        std::process::exit(2);
    };

    dioxus_tui::render(RenderRequest::new(spec.app).with_config(spec.cfg)).unwrap();
}
