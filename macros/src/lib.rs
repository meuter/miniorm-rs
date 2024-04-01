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
#[proc_macro_derive(Entity, attributes(sqlx, column, postgres, sqlite, mysql))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    use Database::*;

    let input: DeriveInput = syn::parse(input).unwrap();
    let args = SchemaArgs::from_derive_input(&input).expect("could not parse args");

    let mut result = quote!();

    if args.columns().any(|col| col.supports_db(&Postgres)) {
        let schema_impl = args.generate_schema_impl(&Postgres);
        let bind_impl = args.generate_bind_impl(&Postgres);
        result = quote! {
            #result
            #schema_impl
            #bind_impl
        }
    }

    if args.columns().any(|col| col.supports_db(&Sqlite)) {
        let schema_impl = args.generate_schema_impl(&Sqlite);
        let bind_impl = args.generate_bind_impl(&Sqlite);
        result = quote! {
            #result
            #schema_impl
            #bind_impl
        }
    }

    if args.columns().any(|col| col.supports_db(&MySql)) {
        let schema_impl = args.generate_schema_impl(&MySql);
        let bind_impl = args.generate_bind_impl(&MySql);
        result = quote! {
            #result
            #schema_impl
            #bind_impl
        }
    }

    result.into()
}
