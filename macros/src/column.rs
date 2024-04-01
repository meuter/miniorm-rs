use std::collections::HashMap;

use darling::FromField;
use quote::quote;
use syn::{Field, Ident, Meta};

use crate::database::Database;

#[derive(Clone, Debug, FromField)]
#[darling(attributes(sqlx))]
struct InnerColumn {
    ident: Option<Ident>,
    #[darling(skip)]
    schema: HashMap<Database, String>,
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
        use Database::*;

        let mut col = InnerColumn::from_field(field)?;

        // manually parse the #[postgres(...)], #[sqlite(...)] and
        // #[mysql(...)] since there does not appear to be any way
        // to do that directly with darling
        for attr in &field.attrs {
            if let Meta::List(list) = &attr.meta {
                let schema = list.tokens.to_string();
                if list.path.is_ident("postgres") {
                    col.schema.insert(Postgres, schema);
                } else if list.path.is_ident("sqlite") {
                    col.schema.insert(Sqlite, schema);
                } else if list.path.is_ident("mysql") {
                    col.schema.insert(MySql, schema);
                } else if list.path.is_ident("column") {
                    col.schema.insert(Postgres, schema.clone());
                    col.schema.insert(Sqlite, schema.clone());
                    col.schema.insert(MySql, schema);
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

    pub fn value(&self) -> proc_macro2::TokenStream {
        let field_ident = self.ident();
        if self.0.json {
            quote!(::serde_json::to_value(&self.#field_ident).unwrap())
        } else {
            quote!(self.#field_ident.clone())
        }
    }

    pub fn supports_db(&self, db: &Database) -> bool {
        self.0.schema.contains_key(db)
    }

    pub fn schema_for_db(&self, db: &Database) -> &String {
        self.0.schema.get(db).as_ref().unwrap_or_else(|| {
            panic!(
                "missing #[{}(...)] declaration for field '{}'",
                db,
                self.ident()
            )
        })
    }

    pub fn skip(&self) -> bool {
        self.0.skip
    }
}
