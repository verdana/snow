use clap::builder::styling::*;
use clap_builder as clap;

/// Returns a `Styles` object with the following styles
/// which are used to style the help message:
pub fn usage_style() -> clap::builder::Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Magenta.on_default())
}