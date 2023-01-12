//! Helper crate for creating nag lints.
//!
//! Usage looks like following:
//! ```rust,no_run
//! // Needed to link into rustc
//! #![feature(rustc_private)]
//!
//! extern crate rustc_lint;
//! extern crate rustc_middle;
//! // Not used directly in this example, but needed for `declare_lints!()` to work.
//! extern crate rustc_session;
//!
//! mod my_cool_lint {
//!     // This is a lint pass. It holds all the lint logic.
//!     pub(super) struct MyCoolLintPass;
//!
//!     // This needed for `nag_toolkit` to know how to create your lint pass.
//!     impl nag_toolkit::NagLint for MyCoolLintPass {
//!         // There's no need to save `tcx`, you'll have access to it later
//!         fn new(_tcx: rustc_middle::ty::TyCtxt) -> Self {
//!             Self
//!         }
//!     }
//!
//!     // You can choose to implement `EarlyLintPass` instead if you want to.
//!     impl<'tcx> rustc_lint::LateLintPass<'tcx> for MyCoolLintPass {
//!         // Implement various `check_` methods here.
//!     }
//! }
//!
//! nag_toolkit::declare_lints!(
//!     // `my_cool_lint` is a name that will be used for `#[allow()]`/`#[deny()]`/etc.
//!     (lint: my_cool_lint(Warn, "An example lint."));
//!     // specify `early` instead of `late`, if you implemented `EarlyLintPass`
//!     (pass: late my_cool_lint::MyCoolLintPass);
//! );
//!
//! # fn main() {
//! // As a result you have a function `register`, which you can pass to `nag_driver`:
//! let _: fn(&rustc_session::Session, &mut rustc_lint::LintStore) = register;
//! // And a `MY_COOL_LINT: Lint` static that can be used inside of lint passes:
//! let _: &rustc_lint::Lint = &MY_COOL_LINT;
//! // It's private by default, but you can put custom visibility before lint name.
//! # }
//! ```

#![feature(rustc_private)]

extern crate rustc_middle;

use rustc_middle::ty::TyCtxt;

#[doc(hidden)]
pub use paste;

/// A lint that can be used with [`declare_lints!()`].
pub trait NagLint {
    fn new(tcx: TyCtxt) -> Self;
}

/// Declare lints and lint passes.
///
/// See crate-level documentation for details.
#[macro_export]
macro_rules! declare_lints {
    ($(($($t:tt)*));*$(;)?) => {
        $($crate::declare_lints!(@declare $($t)*);)*

        pub fn register(_sess: &::rustc_session::Session, store: &mut ::rustc_lint::LintStore) {
            $($crate::declare_lints!(@register store $($t)*);)*
        }
    };
    (@declare lint: $v:vis $name:ident($level:ident, $description:literal)) => {
        $crate::paste::paste! {
            $v static [< $name:upper >]: ::rustc_lint::Lint = ::rustc_lint::Lint {
                name: stringify!($name),
                default_level: ::rustc_lint::Level::$level,
                desc: $description,
                edition_lint_opts: None,
                report_in_external_macro: false,
                future_incompatible: None,
                is_plugin: false,
                feature_gate: None,
                crate_level_only: false,
            };
        }
    };
    (@declare pass: $kind:ident $name:path) => {
        impl ::rustc_lint::LintPass for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }
        }
    };
    (@register $store:ident lint: $v:vis $name:ident($_level:ident, $_description:literal)) => {
        $crate::paste::paste! {
            $store.register_lints(&[&[< $name:upper >]]);
        }
    };
    (@register $store:ident pass: $kind:ident $name:path) => {
        $crate::paste::paste! {
            $store.[< register_ $kind _pass >](
                |tcx| Box::new(<$name as $crate::NagLint>::new(tcx))
            );
        }
    };
}
