use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass, LintContext as _};
use rustc_middle::ty::TyKind;

nag_toolkit::declare_lints!(
    (lint: option_take_used(Warn, "Option::take() was declared bad for no particular reason"));
    (pass: late OptionTakeDetector);
);

struct OptionTakeDetector;

impl nag_toolkit::NagLint for OptionTakeDetector {
    fn new(_tcx: rustc_middle::ty::TyCtxt) -> Self {
        Self
    }
}

impl<'tcx> LateLintPass<'tcx> for OptionTakeDetector {
    fn check_expr(&mut self, ctx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let typeck_results = ctx.typeck_results();
        if let hir::ExprKind::MethodCall(method_name, receiver, _args, span) = expr.kind {
            if method_name.ident.name.as_str() != "take" {
                return;
            }
            let receiver_ty = typeck_results.node_type(receiver.hir_id);
            if let TyKind::Adt(def, _substs) = receiver_ty.kind() {
                let path = ctx.get_def_path(def.did());
                for (expected, found) in ["core", "option", "Option"].into_iter().zip(path) {
                    if expected != found.as_str() {
                        return;
                    }
                }

                ctx.struct_span_lint(
                    &OPTION_TAKE_USED,
                    span,
                    "Option::take() was declared bad for no particular reason",
                    |builder| builder,
                );
            }
        }
    }
}
