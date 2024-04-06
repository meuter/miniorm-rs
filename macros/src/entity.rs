use darling::{ast::Data, FromDeriveInput};
use itertools::Itertools;
use quote::quote;
use syn::Ident;

use crate::{column::Column, database::Database};

#[derive(FromDeriveInput)]
#[darling(attributes(sqlx), supports(struct_named))]
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

    pub fn generate_schema_impl(&self, db: &Database) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let table = self.table_name();
        let cols = self.columns().map(|col| col.name()).join(",");
        let col_name = self.columns().map(|col| col.name());

        // Table
        let create_table = {
            let id_declaration = db.id_declaration();
            let col_declarations = self
                .columns()
                .map(|col| format!("{} {}", col.name(), col.schema_for_db(db)))
                .join(", ");
            format!("CREATE TABLE IF NOT EXISTS {table} ({id_declaration}, {col_declarations})")
        };
        let drop_table = format!("DROP TABLE IF EXISTS {table}");

        // Create
        let create = {
            let placeholders = self
                .columns()
                .enumerate()
                .map(|(i, _)| format!("{}", db.placeholder(i + 1)))
                .join(", ");
            let suffix = match db {
                Database::Postgres | Database::Sqlite => "RETURNING id",
                Database::MySql => "",
            };
            format!("INSERT INTO {table} ({cols}) VALUES ({placeholders}) {suffix}")
        };

        // Read
        let read = format!(
            "SELECT {cols}, id FROM {table} WHERE id={}",
            db.placeholder(1)
        );
        let list = format!("SELECT {cols}, id FROM {table} ORDER BY id");

        // Update
        let update = {
            let id = db.placeholder(self.columns().count() + 1);
            let values = self
                .columns()
                .enumerate()
                .map(|(i, col)| format!("{}={}", col.name(), db.placeholder(i + 1)))
                .join(", ");
            format!("UPDATE {table} SET {values} WHERE id={id}")
        };

        // Datate
        let delete = format!("DELETE FROM {table} WHERE id={}", db.placeholder(1));
        let delete_all = format!("DELETE FROM {table}");

        let db = db.to_token_stream();
        quote! {
            impl ::miniorm::prelude::Schema<#db> for #ident {
                const MINIORM_CREATE_TABLE: &'static str = #create_table;
                const MINIORM_DROP_TABLE: &'static str = #drop_table;
                const MINIORM_CREATE: &'static str = #create;
                const MINIORM_READ: &'static str = #read;
                const MINIORM_LIST: &'static str = #list;
                const MINIORM_UPDATE: &'static str = #update;
                const MINIORM_DELETE: &'static str = #delete;
                const MINIORM_DELETE_ALL: &'static str = #delete_all;
                const MINIORM_TABLE_NAME: &'static str = #table;
                const MINIORM_COLUMNS: &'static [&'static str] = &[
                    #(#col_name,)*
                ];
            }
        }
    }

    pub fn generate_bind_col_impl(&self, db: &Database) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let col_name = self.columns().map(|col| col.name());
        let col_value = self.columns().map(|col| col.value());
        let db = db.to_token_stream();

        quote! {
            impl ::miniorm::prelude::BindColumn<#db> for #ident {
                fn bind_column<'q, Q>(&self, query: Q, column_name: &'static str) -> Q
                where
                    Q: ::miniorm::prelude::Bind<'q, #db> {
                    match column_name {
                        #(#col_name => query.bind(#col_value),)*
                        _ => query,
                    }
                }
            }
        }
    }
}
