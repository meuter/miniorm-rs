use quote::{format_ident, quote};

#[derive(Clone, Debug)]
pub enum Database {
    Postgres,
    Sqlite,
}

impl Database {
    pub fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let ident = format!("{self:#?}");
        let ident = format_ident!("{}", ident);
        quote!(sqlx::#ident)
    }

    pub fn id_declaration(&self) -> &str {
        match self {
            Database::Postgres => "id BIGSERIAL PRIMARY KEY",
            Database::Sqlite => "id INTEGER PRIMARY KEY AUTOINCREMENT",
        }
    }
}
