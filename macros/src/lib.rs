use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, DeriveInput};

fn generate_bind_implementation(input: DeriveInput) -> TokenStream {
    let ident = input.ident;

    quote! {
        impl crate::miniorm::Bind for #ident {
            fn bind<'q, O>(
                &self,
                query: crate::miniorm::PgQueryAs<'q, O>,
                column_name: crate::miniorm::ColunmName
            ) -> crate::miniorm::PgQueryAs<'q, O> {
                match column_name {
                    "date" => query.bind(self.date),
                    "operation" => query.bind(::serde_json::to_value(self.operation).unwrap()),
                    "instrument" => query.bind(::serde_json::to_value(&self.instrument).unwrap()),
                    "quantity" => query.bind(self.quantity),
                    "unit_price" => query.bind(self.unit_price),
                    "taxes" => query.bind(self.taxes),
                    "fees" => query.bind(self.fees),
                    "currency" => query.bind(::serde_json::to_value(self.currency).unwrap()),
                    "exchange_rate" => query.bind(self.exchange_rate),
                    _ => query,
                }
            }

        }
    }
    .into()
}

#[proc_macro_derive(Bind)]
pub fn derive_bind(input: TokenStream) -> TokenStream {
    generate_bind_implementation(parse(input).unwrap())
}
