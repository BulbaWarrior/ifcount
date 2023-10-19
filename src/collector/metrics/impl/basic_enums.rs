use super::prelude::*;
use util::{Hist, Monoid};

#[derive(Default, Serialize, Clone)]
struct Enums {
    variant_count: Hist,
    attr_count: Hist,
    variant_attr_count: Hist,
}

impl Monoid for Enums {
    fn init() -> Self {
        Self::default()
    }

    fn unite(self, rhs: Self) -> Self {
        Self {
            variant_count: self.variant_count + rhs.variant_count,
            attr_count: self.attr_count + rhs.attr_count,
            variant_attr_count: self.variant_attr_count + rhs.variant_attr_count,
        }
    }
}

impl Visit<'_> for Enums {
    fn visit_item_enum(&mut self, i: &'_ syn::ItemEnum) {
        self.variant_count.observe(i.variants.len());
        self.attr_count.observe(i.attrs.len());
        i.variants
            .iter()
            .for_each(|x| self.variant_attr_count.observe(x.attrs.len()));
        syn::visit::visit_item_enum(self, i);
    }
}

pub fn make_collector() -> MetricCollectorBox {
    util::VisitorCollector::new(
        "enums",
        Enums::default(),
        |v| v,
        |v| Monoid::reduce(v.iter().cloned()),
    )
    .make_box()
}

#[cfg(test)]
mod tests {
    use crate::collector::metrics::r#impl::basic_enums::Enums;
    use crate::collector::metrics::util::check;
    use expect_test::expect;
    use syn::parse_quote;

    #[test]
    fn no_enums() {
        let code = parse_quote! {
            struct Thing {
                u: i32,
            }
        };
        check::<Enums>(
            code,
            expect![[r#"
            {
              "variant_count": {
                "sum": 0,
                "avg": null,
                "mode": null
              },
              "attr_count": {
                "sum": 0,
                "avg": null,
                "mode": null
              },
              "variant_attr_count": {
                "sum": 0,
                "avg": null,
                "mode": null
              }
            }"#]],
        );
    }

    #[test]
    fn small_enums() {
        let code = parse_quote! {
            #[derive(Debug, Clone, Copy)]
            enum SmallEnum {
                A,
                B,
                C,
            }

            #[derive(Debug, Clone)]
            enum SmallEnum {
                A,
                B,
            }
        };
        check::<Enums>(
            code,
            expect![[r#"
            {
              "variant_count": {
                "sum": 5,
                "avg": 2.5,
                "mode": 3
              },
              "attr_count": {
                "sum": 2,
                "avg": 1.0,
                "mode": 1
              },
              "variant_attr_count": {
                "sum": 0,
                "avg": 0.0,
                "mode": 0
              }
            }"#]],
        );
    }

    #[test]
    fn big_enum() {
        let code = parse_quote! {
            #[derive(Debug, Clone, Copy)]
            #[serde(tag = "type")]
            enum BigEnum {
                #[serde(rename = "a")]
                A,
                #[serde(rename = "b")]
                B,
                C,
                D,
                E,
                F
            }
        };
        check::<Enums>(
            code,
            expect![[r#"
            {
              "variant_count": {
                "sum": 6,
                "avg": 6.0,
                "mode": 6
              },
              "attr_count": {
                "sum": 2,
                "avg": 2.0,
                "mode": 2
              },
              "variant_attr_count": {
                "sum": 2,
                "avg": 0.3333333333333333,
                "mode": 0
              }
            }"#]],
        );
    }
}
