#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
//! Helper crate for `miniorm` providing a device macro to easily
//! implement the `Schema` and `Bind` trait.
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
///
/// # The `column` directive
///
/// The schema for the column can provided using the `column` directive:
///
/// ```rust
/// use miniorm::prelude::*;
/// use sqlx::FromRow;
///
/// #[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity)]
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
/// supported database types.
/// <table>
///     <tr>
///         <td style="background-color:green;color:black;">
///         Requires the <span style="color:blue">full</span> feature flag.
///         </td>
///     </tr>
/// </table>
///
/// # The backend-specific directive
///
/// If only a specific backend is necessary, one of the dedicated backend-specific
/// directive should be used instead. For instance, if `Schema` and `Bind` should
/// only be derived for `postgres`:
///
/// ```rust
/// use miniorm::prelude::*;
/// use sqlx::FromRow;
///
/// #[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity)]
/// struct Todo {
///     #[postgres(TEXT NOT NULL)]
///     description: String,
///
///     #[postgres(BOOLEAN NOT NULL DEFAULT false)]
///     done: bool,
/// }
/// ```
/// <table>
///     <tr>
///         <td style="background-color:green;color:black;">
///         This example requires the <span style="color:blue">postgres</span> feature flag.
///         </td>
///     </tr>
/// </table>
///
/// # The `sqlx` directive
///
/// At the moment, only the following `sqlx` directives for `FromRow` are supported:
/// - `skip`
/// - `rename`
/// - `json`
///
/// ```rust
/// use miniorm::prelude::*;
/// use sqlx::FromRow;
///
/// #[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity)]
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
