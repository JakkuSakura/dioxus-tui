use std::env;

pub fn dims_from_args_env_or_default(default_width: u16, default_height: u16) -> (u16, u16) {
    let mut args = env::args().skip(1);

    let width = args
        .next()
        .and_then(|s| s.parse::<u16>().ok())
        .or_else(|| env::var("COLUMNS").ok().and_then(|s| s.parse::<u16>().ok()))
        .unwrap_or(default_width);

    let height = args
        .next()
        .and_then(|s| s.parse::<u16>().ok())
        .or_else(|| env::var("LINES").ok().and_then(|s| s.parse::<u16>().ok()))
        .unwrap_or(default_height);

    (width.max(1), height.max(1))
}
