#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../README.md"))]

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
/// The schema for the column can provided using the `column` directive:
///
/// ```rust
/// #[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity, Serialize, Deserialize)]
/// struct Todo {
///     #[column(TEXT NOT NULL)]
///     description: String,
///
///     #[column(BOOLEAN NOT NULL DEFAULT false)]
///     done: bool,
/// }
/// ```
///
/// in which case the `Schema` and `Bind` trait will be derived for all
/// supported database types. This requires the `full` feature of `miniorm`.
///
/// If only a specific backend is necessary, one of the dedicated backend-specific
/// directive should be used instead. For instance, if `Schema` and `Bind` should
/// only be derived for `postgres`:
///
/// ```rust
/// #[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity, Serialize, Deserialize)]
/// struct Todo {
///     #[postgres(TEXT NOT NULL)]
///     description: String,
///
///     #[postgres(BOOLEAN NOT NULL DEFAULT false)]
///     done: bool,
/// }
/// ```
/// At the moment, only the following `sqlx` directives for `FromRow` are supported:
/// - `skip`
/// - `rename`
/// - `json`
///
/// ```rust
/// #[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity, Serialize, Deserialize)]
/// struct Todo {
///     #[postgres(TEXT NOT NULL)]
///     description: String,
///
///     #[postgres(BOOLEAN NOT NULL DEFAULT false)]
///     #[sqlx(rename = "DONE")]
///     done: bool,
///
///     #[sqlx(skip)]
///     metadata: String,
/// }
/// ```
///
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
