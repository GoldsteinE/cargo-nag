#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_session;

use std::{env, io, mem, process::ExitCode};

use rustc_lint::LintStore;
use rustc_session::Session;

type BoxCallback = Box<dyn Fn(&Session, &mut LintStore) + Send + Sync>;

/// The entry point for a custom linter.
///
/// Use it like this:
/// ```rust,no_run
/// # #![feature(rustc_private)]
/// # extern crate rustc_lint;
/// # extern crate rustc_session;
/// # mod lints {
/// #     pub fn register(_sess: &rustc_session::Session, _store: &mut rustc_lint::LintStore) {}
/// # }
/// # mod external_lints {
/// #     pub fn register(_sess: &rustc_session::Session, _store: &mut rustc_lint::LintStore) {}
/// # }
/// # use std::process::ExitCode;
/// # use nag_driver::Driver;
/// fn main() -> ExitCode {
///     Driver::new()
///         .with_cfg("nag")
///         .with_callbacks([
///             lints::register,
///             external_lints::register,
///         ])
///         .run()
/// }
/// ```
#[derive(Default)]
pub struct Driver {
    cfg: Vec<String>,
    callbacks: Vec<BoxCallback>,
}

impl Driver {
    /// Create a new [`Driver`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `--cfg={cfg}` flag to the compiler invocation.
    ///
    /// This allows for the following pattern:
    /// ```rust
    /// #[cfg_attr(nag, allow(custom_lint))]
    /// fn func() {
    ///     // ...
    /// }
    /// ```
    pub fn with_cfg(mut self, cfg: impl Into<String>) -> Self {
        self.cfg.push(cfg.into());
        self
    }

    /// Add a single callback for registering lints.
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Session, &mut LintStore) + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
        self
    }

    /// Add multiple callbacks for registering lints.
    pub fn with_callbacks<F, I>(mut self, callbacks: I) -> Self
    where
        F: Fn(&Session, &mut LintStore) + Send + Sync + 'static,
        I: IntoIterator<Item = F>,
    {
        self.callbacks
            .extend(callbacks.into_iter().map(|cb| Box::new(cb) as _));
        self
    }

    /// Run the driver with args from the [`std::env::args`]. This is probably what you want.
    #[must_use]
    pub fn run(self) -> ExitCode {
        self.run_with_args(env::args().skip(1))
    }

    /// Run the driver with custom `args`.
    ///
    /// *Note*: `argv[0]`, i.e. the executable name (`rustc`), should be passed into this function.
    #[must_use]
    pub fn run_with_args(mut self, args: impl IntoIterator<Item = String>) -> ExitCode {
        if env::var("CARGO_NAG_DUMP_EXECUTION_PARAMS").is_ok() {
            dump_execution_params();
            return ExitCode::SUCCESS;
        }

        let mut args: Vec<_> = args.into_iter().collect();
        for cfg in &self.cfg {
            args.push(format!("--cfg={cfg}"));
        }

        if let Ok(sysroot) = env::var("CARGO_NAG_SYSROOT") {
            args.push(format!("--sysroot={sysroot}"));
        }

        if rustc_driver::RunCompiler::new(&args, &mut self)
            .run()
            .is_err()
        {
            return ExitCode::FAILURE;
        }

        ExitCode::SUCCESS
    }
}

/// Shortcut for simple cases
///
/// Equivalent to
///
/// ```rust,no_run
/// # #![feature(rustc_private)]
/// # extern crate rustc_lint;
/// # extern crate rustc_session;
/// # use std::process::ExitCode;
/// # use nag_driver::Driver;
/// # fn register(_sess: &rustc_session::Session, _store: &mut rustc_lint::LintStore) {}
/// # fn main() -> ExitCode {
/// # let cfg = "nag";
/// # let callbacks = [register];
/// Driver::new()
///     .with_cfg(cfg)
///     .with_callbacks(callbacks)
///     .run()
/// # }
#[must_use]
pub fn run<F, I, C>(cfg: C, callbacks: I) -> ExitCode
where
    F: Fn(&Session, &mut LintStore) + Send + Sync + 'static,
    I: IntoIterator<Item = F>,
    C: Into<String>,
{
    Driver::new().with_cfg(cfg).with_callbacks(callbacks).run()
}

/// This implementation will clear the callbacks list stored inside of [`Driver`].
///
/// You probably should use [`Driver::run()`] or [`Driver::run_with_args()`] instead.
impl rustc_driver::Callbacks for Driver {
    fn config(&mut self, config: &mut rustc_interface::Config) {
        let previous = config.register_lints.take();
        let callbacks = mem::take(&mut self.callbacks);
        config.register_lints = Some(Box::new(move |sess, lint_store| {
            if let Some(previous) = &previous {
                previous(sess, lint_store);
            }

            for callback in &callbacks {
                callback(sess, lint_store);
            }
        }));
    }
}

fn dump_execution_params() {
    #[derive(serde::Serialize)]
    struct ExecutionParams {
        binary: String,
        environment: std::collections::HashMap<String, String>,
    }

    let environment = env::vars().collect();
    let binary = env::args().next().expect("no binary name in argv");
    serde_json::to_writer(
        io::stdout().lock(),
        &ExecutionParams {
            binary,
            environment,
        },
    )
    .expect("failed to write execution params to stdout");
}
