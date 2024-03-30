mod column;
mod database;
mod entity;

use darling::FromDeriveInput;
use database::Database;
use entity::SchemaArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Derive macro to automatically derive the `Schema` trait.
#[proc_macro_derive(Schema, attributes(miniorm, sqlx, column, postgres, sqlite))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let args = SchemaArgs::from_derive_input(&input).expect("could not parse args");

    let mut result = quote!();

    if args.columns().any(|col| col.has_postgres()) {
        let postgres_impl = args.generate_schema_impl(Database::Postgres);
        result = quote! {
            #result
            #postgres_impl
        }
    }

    if args.columns().any(|col| col.has_sqlite()) {
        let sqlite_impl = args.generate_schema_impl(Database::Sqlite);
        result = quote! {
            #result
            #sqlite_impl
        }
    }

    result.into()
}
