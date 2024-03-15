use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, DeriveInput, Meta, MetaList};

fn generate_bind_implementation(input: DeriveInput) -> TokenStream {
    let ident = input.ident;

    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        syn::Data::Enum(_) => panic!("only structs are supported"),
        syn::Data::Union(_) => panic!("only structs are supported"),
    };

    let match_arms = fields.into_iter().map(|field| {
        let field_ident = field.ident.unwrap();
        let field_str = field_ident.to_string();

        let is_sqlx_json = field.attrs.into_iter().any(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                let MetaList { path, tokens, .. } = meta_list;
                let is_sqlx = path.segments.iter().any(|seg| seg.ident == "sqlx");
                let is_json = tokens.to_string() == "json";
                is_sqlx && is_json
            } else {
                false
            }
        });

        if is_sqlx_json {
            quote! {
                #field_str => query.bind(::serde_json::to_value(&self.#field_ident).unwrap()),
            }
        } else {
            quote! {
                #field_str => query.bind(self.#field_ident.clone()),
            }
        }
    });

    quote! {
        impl ::miniorm::Bind for #ident {
            fn bind<'q, O>(
                &self,
                query: ::miniorm::PgQueryAs<'q, O>,
                column_name: ::miniorm::ColunmName
            ) -> ::miniorm::PgQueryAs<'q, O> {
                match column_name {
                    #(#match_arms)*
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
