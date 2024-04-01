use quote::{format_ident, quote};
use std::borrow::Cow;
use strum::{Display, EnumIter};

#[derive(Debug, Hash, Clone, Eq, PartialEq, EnumIter, Display)]
pub enum Database {
    Postgres,
    Sqlite,
    MySql,
}

impl Database {
    pub fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let ident = format!("{self:#?}");
        let ident = format_ident!("{}", ident);
        quote!(sqlx::#ident)
    }

    pub fn id_declaration(&self) -> &str {
        use Database::*;
        match self {
            Postgres => "id BIGSERIAL PRIMARY KEY",
            Sqlite => "id INTEGER PRIMARY KEY AUTOINCREMENT",
            MySql => "id INT AUTO_INCREMENT NOT NULL PRIMARY KEY",
        }
    }

    pub fn placeholder(&self, index: usize) -> Cow<'_, str> {
        use Database::*;
        match self {
            Postgres | Sqlite => format!("${index}").into(),
            MySql => "?".into(),
        }
    }
}
