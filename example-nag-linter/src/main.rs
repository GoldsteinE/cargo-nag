use std::process::ExitCode;

use example_nag_linter::lints;

fn main() -> ExitCode {
    nag_driver::run("nag", [lints::register])
}
