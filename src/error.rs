use anyhow::Error as AnyError;
use thiserror::Error;
use termwiz::Error as TermwizError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("terminal pixel dimensions unavailable (xpixel={xpixel}, ypixel={ypixel})")]
    TerminalPixelsUnavailable { xpixel: usize, ypixel: usize },
    #[error(transparent)]
    Termwiz(#[from] TermwizError),
    #[error(transparent)]
    Other(#[from] AnyError),
}

pub type Result<T> = std::result::Result<T, Error>;
