use darling::{ast::Data, FromDeriveInput};
use quote::quote;
use syn::Ident;

use crate::{column::Column, database::Database};

#[derive(FromDeriveInput)]
#[darling(attributes(miniorm, sqlx), supports(struct_named))]
pub struct SchemaArgs {
    ident: Ident,
    rename: Option<String>,
    data: Data<(), Column>,
}

impl SchemaArgs {
    fn table_name(&self) -> String {
        self.rename
            .as_ref()
            .cloned()
            .unwrap_or(self.ident.to_string().to_lowercase())
    }

    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        match &self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields.fields.iter().filter(|col| !col.skip()),
        }
    }

    pub fn generate_schema_impl(&self, db: Database) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let table_name = self.table_name();
        let id_declaration = db.id_declaration();

        let field_str = self.columns().map(|col| col.name());
        let col_type = self.columns().map(|col| match db {
            Database::Postgres => col.postgres(),
            Database::Sqlite => col.sqlite(),
        });

        let value = self.columns().map(|col| {
            let field_ident = col.ident();
            if col.json() {
                quote!(::serde_json::to_value(&self.#field_ident).unwrap())
            } else {
                quote!(self.#field_ident.clone())
            }
        });
        let field_str2 = self.columns().map(|col| col.name());

        let db = db.to_token_stream();

        quote! {
            impl ::miniorm::traits::Schema<#db> for #ident {
                const ID_DECLARATION: &'static str = #id_declaration;
                const TABLE_NAME: &'static str = #table_name;
                const COLUMNS: &'static [(&'static str, &'static str)] = &[
                    #((#field_str, #col_type),)*
                ];

                fn bind<'q, O>(
                    &self,
                    query: ::miniorm::traits::QueryAs<'q, #db, O>,
                    column_name: &'static str
                ) -> ::miniorm::traits::QueryAs<'q, #db, O> {
                    match column_name {
                        #(#field_str2 => query.bind(#value),)*
                        _ => query,
                    }
                }
            }

        }
    }
}
