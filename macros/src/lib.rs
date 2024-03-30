mod column;
mod database;
mod entity;

use darling::FromDeriveInput;
use database::Database;
use entity::SchemaArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Derive macro to automatically derive the `Schema` and `Bind` traits.
#[proc_macro_derive(Entity, attributes(miniorm, sqlx, column, postgres, sqlite))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let args = SchemaArgs::from_derive_input(&input).expect("could not parse args");

    let mut result = quote!();

    if args.columns().any(|col| col.has_postgres()) {
        let schema_impl = args.generate_schema_impl(Database::Postgres);
        let bind_impl = args.generate_bind_impl(Database::Postgres);
        result = quote! {
            #result
            #schema_impl
            #bind_impl
        }
    }

    if args.columns().any(|col| col.has_sqlite()) {
        let schema_impl = args.generate_schema_impl(Database::Sqlite);
        let bind_impl = args.generate_bind_impl(Database::Sqlite);
        result = quote! {
            #result
            #schema_impl
            #bind_impl
        }
    }

    result.into()
}
