mod column;
mod database;
mod entity;

use darling::FromDeriveInput;
use database::Database;
use entity::SchemaArgs;
use proc_macro::TokenStream;
use quote::quote;
use strum::IntoEnumIterator;
use syn::DeriveInput;

/// Derive macro to automatically derive the `Schema` and `Bind` traits.
#[proc_macro_derive(Entity, attributes(sqlx, column, postgres, sqlite, mysql))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let args = SchemaArgs::from_derive_input(&input).expect("could not parse args");

    let mut result = quote!();

    for db in Database::iter() {
        if args.columns().any(|col| col.supports_db(&db)) {
            let schema_impl = args.generate_schema_impl(&db);
            let bind_impl = args.generate_bind_col_impl(&db);
            result = quote! {
                #result
                #schema_impl
                #bind_impl
            }
        }
    }

    result.into()
}
