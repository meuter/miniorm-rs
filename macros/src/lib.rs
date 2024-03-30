use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Meta, MetaList};

/// Derive macro to automatically derive the `Schema` trait.
#[proc_macro_derive(Schema, attributes(column))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let ident = input.ident;
    let table_name = ident.to_string().to_lowercase();

    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        syn::Data::Enum(_) => panic!("only structs are supported"),
        syn::Data::Union(_) => panic!("only structs are supported"),
    };

    let columns = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_str = field_ident.to_string();

        let col_type = field
            .attrs
            .iter()
            .filter_map(|attr| {
                if let Meta::List(meta_list) = &attr.meta {
                    let MetaList { path, tokens, .. } = meta_list;
                    let is_column = path.segments.iter().any(|seg| seg.ident == "column");
                    if is_column {
                        let col_type = tokens.to_string();
                        Some(col_type)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| panic!("missing `#[column(<type>)]` for `{field_str}`, e.g. `#[column(TEXT NOT NULL)]`"));

        quote! {
            (#field_str, #col_type),
        }
    });

    let bind_match_arms = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_str = field_ident.to_string();

        let is_sqlx_json = field.attrs.iter().any(|attr| {
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
        impl ::miniorm::traits::Schema<sqlx::Postgres> for #ident {

            const TABLE_NAME: &'static str = #table_name;

            const COLUMNS: &'static [(&'static str, &'static str)] = &[
                #(#columns)*
            ];

            fn bind<'q, O>(
                &self,
                query: ::miniorm::traits::QueryAs<'q, sqlx::Postgres, O>,
                column_name: &'static str
            ) -> ::miniorm::traits::QueryAs<'q, sqlx::Postgres, O> {
                match column_name {
                    #(#bind_match_arms)*
                    _ => query,
                }
            }
        }
    }
    .into()
}
