use syn::visit::{self, Visit};

use super::prelude::*;
use util::{Monoid, Unaggregated};

#[derive(Default)]
pub struct StatementSize {
    expr_count: usize,
    hist: Unaggregated,
}

impl Visit<'_> for StatementSize {
    fn visit_expr(&mut self, i: &'_ syn::Expr) {
        self.expr_count += 1;

        visit::visit_expr(self, i);
    }

    fn visit_stmt(&mut self, i: &'_ syn::Stmt) {
        let old_expr_count = self.expr_count;
        self.expr_count = 0;

        visit::visit_stmt(self, i);

        self.hist.observe(self.expr_count);
        self.expr_count = old_expr_count;
    }
}

pub fn make_collector() -> MetricCollectorBox {
    util::VisitorCollector::new(
        "statement_size",
        StatementSize::default(),
        |v| v,
        |v: &[StatementSize]| Monoid::reduce(v.iter().map(|v| v.hist.to_owned())),
    )
    .make_box()
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};
    use syn::{parse_quote, visit::Visit, File};

    use super::StatementSize;

    fn check(code: File, expect: Expect) {
        let mut statements = StatementSize::default();
        statements.visit_file(&code);
        let actual = serde_json::to_string_pretty(&statements.hist).unwrap();
        expect.assert_eq(&actual);
    }

    #[test]
    fn mut_ident() {
        let code: File = parse_quote! {
            fn main() {
                // Note: function name and function invokation
                // are separete expressions
                //
                // 1 statement with 7 expressions
                //                    6 5        4 3 2   1
                let mut aboba = Arc::new(Mutex::new(Some(69))); // whole assignment 7
            }
        };
        dbg!(&code);
        check(
            code,
            expect![[r#"
            {
              "sum": 7,
              "avg": 7.0,
              "mode": 7
            }"#]],
        );
    }

    #[test]
    fn if_let_pat() {
        let code: File = parse_quote! {
            fn main() {
                // Note: function name and function invokation
                // are separete expressions
                //
                // 2 statements with 3 and 1 expressions
                //               2  1
                let Some(val) = bar() else { // 3
                    panic!() // second statement which is an expression
                };
            }
        };
        dbg!(&code);
        check(
            code,
            expect![[r#"
            {
              "sum": 4,
              "avg": 2.0,
              "mode": 3
            }"#]],
        )
    }
}
