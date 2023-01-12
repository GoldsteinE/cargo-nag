# cargo-nag: experimental toolkit for writing custom Rust linters

## How to write lints?

Use [nag-toolkit] library, call `declare_lints!()` macro and export resulting `::register()` function: see <example-nag-linter/src/lints.rs> as an example.

You can provide a crate with lints for the users of your library or just write them in the same crate as your linter.

[nightly-rustc] documentation, especially part about [LateLintPass] is extremely useful when writing lints.

## How to write a linter?

Use [nag-driver] library, call `nag_driver::run()` or `nag_driver::Driver::with_callback()` and then `.run()`: see <example-nag-linter/src/main.rs> as an example.

## How to use a linter?

Install `cargo-nag` binary. Set `CARGO_NAG_LINTER_DIR` environment variable to the absolute path to your linter source code, then run `cargo nag`.
It will compile and run a linter for you.

[nag-toolkit]: https://docs.rs/nag-toolkit
[nag-driver]: https://docs.rs/nag-driver
[nightly-rustc]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/trait.LateLintPass.html
[LateLintPass]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/trait.LateLintPass.html
