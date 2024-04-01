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
                let placeholders = self
                    .columns()
                    .enumerate()
                    .map(|(i, _)| format!("${}", i + 1))
                    .join(", ");
                format!("INSERT INTO {table} ({cols}) VALUES ({placeholders}) RETURNING id")
            }
            Database::MySql => {
                // MySql uses `?` as placeholders and does not support `RETURNING id`
                let placeholders = self.columns().map(|_| "?").join(", ");
                format!("INSERT INTO {table} ({cols}) VALUES ({placeholders})")
            }
        }
    }

    fn read(&self, db: Database) -> String {
        let table = self.table_name();
        let cols = self.columns().map(|col| col.name()).join(", ");
        match db {
            Database::Postgres | Database::Sqlite => {
                format!("SELECT {cols} FROM {table} WHERE id=$1")
            }
            Database::MySql => format!("SELECT {cols} FROM {table} WHERE id=?"),
        }
    }

    fn list(&self) -> String {
        let table = self.table_name();
        let cols = self.columns().map(|col| col.name()).join(", ");
        format!("SELECT {cols} FROM {table} ORDER BY id")
    }

    fn update(&self, db: Database) -> String {
        let table = self.table_name();
        let id = self.columns().count() + 1;
        match db {
            Database::Postgres | Database::Sqlite => {
                let values = self
                    .columns()
                    .enumerate()
                    .map(|(i, col)| format!("{}=${}", col.name(), i + 1))
                    .join(", ");
                format!("UPDATE {table} SET {values} WHERE id=${id}")
            }
            Database::MySql => {
                let values = self
                    .columns()
                    .map(|col| format!("{}=?", col.name()))
                    .join(", ");
                format!("UPDATE {table} SET {values} WHERE id=?")
            }
        }
    }

    fn delete(&self, db: Database) -> String {
        let table = self.table_name();
        match db {
            Database::Postgres | Database::Sqlite => format!("DELETE FROM {table} WHERE id=$1"),
            Database::MySql => format!("DELETE FROM {table} WHERE id=?"),
        }
    }

    fn delete_all(&self) -> String {
        let table = self.table_name();
        format!("DELETE FROM {table}")
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
        let read = self.read(db);
        let list = self.list();
        let update = self.update(db);
        let delete = self.delete(db);
        let delete_all = self.delete_all();

        let db = db.to_token_stream();
        quote! {
            impl ::miniorm::Schema<#db> for #ident {
                const MINIORM_CREATE_TABLE: &'static str = #create_table;
                const MINIORM_DROP_TABLE: &'static str = #drop_table;
                const MINIORM_CREATE: &'static str = #create;
                const MINIORM_READ: &'static str = #read;
                const MINIORM_LIST: &'static str = #list;
                const MINIORM_UPDATE: &'static str = #update;
                const MINIORM_DELETE: &'static str = #delete;
                const MINIORM_DELETE_ALL: &'static str = #delete_all;
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
