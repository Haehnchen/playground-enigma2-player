mod desktop;
mod main;

#[doc(hidden)]
pub use desktop::quote_desktop_path;
pub use main::run_from_env;
#[doc(hidden)]
pub use main::{parse_args, CliOptions};
