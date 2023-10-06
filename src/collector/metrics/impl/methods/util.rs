use std::collections::{HashMap, HashSet};

use syn::{parse_quote, Ident, ImplItemFn};

use super::prelude::*;

#[derive(Default, Debug)]
pub struct FieldSet(pub HashSet<syn::Member>);

#[derive(Default, Debug)]
pub struct FnFieldSet {
    pub field_usage: HashMap<ImplItemFn, FieldSet>,
}

impl FnFieldSet {
    pub fn related(&self, func1: &ImplItemFn, func2: &ImplItemFn) -> bool {
        let field_set1 = self.field_usage.get(func1).unwrap().0.clone();
        let field_set2 = self.field_usage.get(func2).unwrap().0.clone();
        field_set1.intersection(&field_set2).count() > 0
    }
}

#[derive(Default, Debug)]
pub struct MethodCalls(pub HashSet<Ident>);

#[derive(Default, Debug)]
pub struct FnMethodCalls(pub HashMap<ImplItemFn, MethodCalls>);

impl FnMethodCalls {
    pub fn related(&self, func1: &ImplItemFn, func2: &ImplItemFn) -> bool {
        self.0.get(func1).unwrap().0.contains(&func2.sig.ident)
            || self.0.get(func2).unwrap().0.contains(&func1.sig.ident)
    }
}
impl Visit<'_> for MethodCalls {
    fn visit_expr_method_call(&mut self, i: &'_ syn::ExprMethodCall) {
        if i.receiver == parse_quote!(self) {
            self.0.insert(i.method.clone());
        }
        syn::visit::visit_expr_method_call(self, i);
    }
}

impl Visit<'_> for FnMethodCalls {
    fn visit_impl_item_fn(&mut self, i: &'_ ImplItemFn) {
        let mut calls = MethodCalls::default();
        calls.visit_impl_item_fn(i);
        self.0.insert(i.clone(), calls);
        syn::visit::visit_impl_item_fn(self, i);
    }
}

impl Visit<'_> for FieldSet {
    fn visit_expr_field(&mut self, i: &'_ syn::ExprField) {
        if i.base == parse_quote!(self) {
            self.0.insert(i.member.clone());
        }
        syn::visit::visit_expr_field(self, i);
    }
}

impl Visit<'_> for FnFieldSet {
    fn visit_impl_item_fn(&mut self, i: &'_ ImplItemFn) {
        let mut fields = FieldSet::default();
        fields.visit_impl_item_fn(i);
        self.field_usage.insert(i.clone(), fields);
        syn::visit::visit_impl_item_fn(self, i);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use syn::{parse_quote, visit::Visit, File, Member};

    use super::FnFieldSet;

    use super::MethodCalls;

    fn code_file() -> File {
        let code = parse_quote! {
            impl Aboba {
                fn hehe(&self) {
                    self.a + self.b + self.aboba_call() // hehe uses a and b,
                                                        // method calls are not fields
                }
                fn haha(&mut self) {
                    self.b + self.c + self.aboba_call() // haha uses b and c
                }
            }
        };
        syn::parse2(code).unwrap()
    }

    #[test]
    fn field_intersection() {
        let syntax_tree = code_file();
        let mut visitor = FnFieldSet::default();
        visitor.visit_file(&syntax_tree);
        dbg!(&visitor);
        let field_sets: Vec<_> = visitor.field_usage.values().collect();
        let intersection: HashSet<_> = field_sets[0]
            .0
            .iter()
            .filter(|x| field_sets.iter().all(|set| set.0.contains(x)))
            .collect();
        dbg!(&intersection);
        let res: Member = parse_quote!(b);
        assert_eq!(intersection, HashSet::from([&res]));
    }

    #[test]
    fn method_calls() {
        let code = parse_quote! {
            fn yeya(self) {
                self.test() * self.zhizha().times(self.field.foo())
            } //       ^              ^                        ^ method of field
        };
        let syntax_tree = syn::parse2(code).unwrap();
        let mut visitor = MethodCalls::default();
        visitor.visit_file(&syntax_tree);
        assert_eq!(
            visitor.0,
            HashSet::from([parse_quote!(zhizha), parse_quote!(test)])
        )
    }
}