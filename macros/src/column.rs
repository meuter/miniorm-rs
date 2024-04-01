use darling::FromField;
use quote::quote;
use syn::{Field, Ident, Meta};

use crate::database::Database;

#[derive(Clone, Debug, FromField)]
#[darling(attributes(miniorm, sqlx))]
struct InnerColumn {
    ident: Option<Ident>,
    postgres: Option<String>,
    sqlite: Option<String>,
    mysql: Option<String>,
    rename: Option<String>,
    #[darling(default)]
    json: bool,
    #[darling(default)]
    skip: bool,
}

#[derive(Debug, Clone)]
pub struct Column(InnerColumn);

impl FromField for Column {
    fn from_field(field: &Field) -> darling::Result<Self> {
        let mut col = InnerColumn::from_field(field)?;

        // manually parse the #[postgres(...)], #[sqlite(...)] and
        // #[mysql(...)] since there does not appear to be any way
        // to do that directly with darling
        for attr in &field.attrs {
            if let Meta::List(list) = &attr.meta {
                let schema = list.tokens.to_string();
                if list.path.is_ident("postgres") {
                    col.postgres = Some(schema);
                } else if list.path.is_ident("sqlite") {
                    col.sqlite = Some(schema)
                } else if list.path.is_ident("mysql") {
                    col.mysql = Some(schema)
                } else if list.path.is_ident("column") {
                    col.sqlite = Some(schema.clone());
                    col.postgres = Some(schema.clone());
                    col.mysql = Some(schema);
                }
            }
        }

        Ok(Self(col))
    }
}

impl Column {
    pub fn ident(&self) -> &Ident {
        self.0.ident.as_ref().unwrap()
    }

    pub fn name(&self) -> String {
        self.0
            .rename
            .as_ref()
            .cloned()
            .unwrap_or(self.ident().to_string())
    }

    pub fn sql_type(&self, db: Database) -> String {
        let col_type = match db {
            Database::Postgres => self.postgres(),
            Database::Sqlite => self.sqlite(),
            Database::MySql => self.mysql(),
        };
        col_type.to_string()
    }

    pub fn value(&self) -> proc_macro2::TokenStream {
        let field_ident = self.ident();
        if self.0.json {
            quote!(::serde_json::to_value(&self.#field_ident).unwrap())
        } else {
            quote!(self.#field_ident.clone())
        }
    }

    pub fn has_postgres(&self) -> bool {
        self.0.postgres.is_some()
    }

    pub fn postgres(&self) -> &String {
        self.0.postgres.as_ref().unwrap_or_else(|| {
            panic!(
                "missing #[postgres(...)] declaration for field '{}'",
                self.ident()
            )
        })
    }

    pub fn has_mysql(&self) -> bool {
        self.0.mysql.is_some()
    }

    pub fn mysql(&self) -> &String {
        self.0.mysql.as_ref().unwrap_or_else(|| {
            panic!(
                "missing #[mysql(...)] declaration for field '{}'",
                self.ident()
            )
        })
    }

    pub fn has_sqlite(&self) -> bool {
        self.0.sqlite.is_some()
    }

    pub fn sqlite(&self) -> &String {
        self.0.sqlite.as_ref().unwrap_or_else(|| {
            panic!(
                "missing #[sqlite(...)] declaration for field '{}'",
                self.ident()
            )
        })
    }

    pub fn skip(&self) -> bool {
        self.0.skip
    }
}
