use darling::{ast::Data, FromDeriveInput};
use itertools::Itertools;
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

    fn create_table(&self, db: Database) -> String {
        let table = self.table_name();
        let id = db.id_declaration();
        let cols = self
            .columns()
            .map(|col| format!("{} {}", col.name(), col.sql_type(db)))
            .join(", ");
        format!("CREATE TABLE IF NOT EXISTS {table} ({id}, {cols})")
    }

    fn drop_table(&self) -> String {
        let table = self.table_name();
        format!("DROP TABLE IF EXISTS {table}")
    }

    fn create(&self, db: Database) -> String {
        let table = self.table_name();
        let cols = self.columns().map(|col| col.name()).join(", ");
        match db {
            Database::Postgres | Database::Sqlite => {
                let placeholders = (1..=self.columns().count())
                    .map(|i| format!("${i}"))
                    .join(", ");
                format!("INSERT INTO {table} ({cols}) VALUES ({placeholders}) RETURNING id")
            }
            Database::MySql => {
                // MySql uses `?` as placeholders and does not support `RETURNING id`
                let placeholders = (1..=self.columns().count()).map(|_| "?").join(", ");
                format!("INSERT INTO {table} ({cols}) VALUES ({placeholders})")
            }
        }
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
        let col_name = self.columns().map(|col| col.name());
        let col_type = self.columns().map(|col| match db {
            Database::Postgres => col.postgres(),
            Database::Sqlite => col.sqlite(),
            Database::MySql => col.mysql(),
        });

        let drop_table = self.drop_table();
        let create_table = self.create_table(db);
        let create = self.create(db);

        let db = db.to_token_stream();
        quote! {
            impl ::miniorm::Schema<#db> for #ident {
                const MINIORM_CREATE_TABLE: &'static str = #create_table;
                const MINIORM_DROP_TABLE: &'static str = #drop_table;
                const MINIORM_CREATE: &'static str = #create;
                const TABLE_NAME: &'static str = #table_name;
                const COLUMNS: &'static [(&'static str, &'static str)] = &[
                    #((#col_name, #col_type),)*
                ];
            }
        }
    }

    pub fn generate_bind_impl(&self, db: Database) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let col_name = self.columns().map(|col| col.name());
        let col_value = self.columns().map(|col| col.value());
        let db = db.to_token_stream();

        quote! {
            impl ::miniorm::Bind<#db> for #ident {
                fn bind<'q, Q>(&self, query: Q, column_name: &'static str) -> Q
                where
                    Q: ::miniorm::BindableQuery<'q, #db> {
                    match column_name {
                        #(#col_name => query.bind(#col_value),)*
                        _ => query,
                    }
                }
            }
        }
    }
}
