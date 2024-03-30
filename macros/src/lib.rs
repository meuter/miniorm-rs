use darling::{ast::Data, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Field, Ident, Meta};

#[derive(Clone, Debug)]
enum Database {
    Postgres,
    Sqlite,
}

impl Database {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let ident = format!("{self:#?}");
        let ident = format_ident!("{}", ident);
        quote!(sqlx::#ident)
    }

    fn id_declaration(&self) -> &str {
        match self {
            Database::Postgres => "id BIGSERIAL PRIMARY KEY",
            Database::Sqlite => "id INTEGER PRIMARY KEY AUTOINCREMENT",
        }
    }
}

#[derive(Clone, Debug, FromField)]
#[darling(attributes(miniorm, sqlx))]
struct SchemaColumn {
    ident: Option<Ident>,
    postgres: Option<String>,
    sqlite: Option<String>,
    #[darling(default)]
    json: bool,
}

#[derive(Debug, Clone)]
struct WrappedSchemaColumn(SchemaColumn);

impl FromField for WrappedSchemaColumn {
    fn from_field(field: &Field) -> darling::Result<Self> {
        let mut col = SchemaColumn::from_field(field)?;

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
                } else if list.path.is_ident("column") {
                    col.sqlite = Some(schema.clone());
                    col.postgres = Some(schema);
                }
            }
        }

        Ok(Self(col))
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(miniorm, sqlx), supports(struct_named))]
struct SchemaArgs {
    ident: Ident,
    rename: Option<String>,
    data: Data<(), WrappedSchemaColumn>,
}

impl SchemaArgs {
    fn table_name(&self) -> String {
        if let Some(name) = &self.rename {
            name.clone()
        } else {
            self.ident.to_string().to_lowercase()
        }
    }

    fn columns(&self) -> impl Iterator<Item = &SchemaColumn> {
        match &self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields.fields.iter().map(|wrapped| &wrapped.0),
        }
    }
}

impl SchemaArgs {
    fn generate_schema_impl(&self, db: Database) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let table_name = self.table_name();

        let id_declaration = db.id_declaration();

        let field_str = self
            .columns()
            .map(|col| col.ident.as_ref().unwrap().to_string());
        let col_type = self.columns().map(|col| match db {
            Database::Postgres => col.postgres.as_ref().expect("missing postgres schema"),
            Database::Sqlite => col.sqlite.as_ref().expect("missing sqlite schema"),
        });

        let value = self.columns().map(|col| {
            let field_ident = col.ident.as_ref().unwrap();
            if col.json {
                quote!(::serde_json::to_value(&self.#field_ident).unwrap())
            } else {
                quote!(self.#field_ident.clone())
            }
        });
        let field_str2 = self
            .columns()
            .map(|col| col.ident.as_ref().unwrap().to_string());

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

/// Derive macro to automatically derive the `Schema` trait.
#[proc_macro_derive(Schema, attributes(miniorm, sqlx, column, postgres, sqlite))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let args = SchemaArgs::from_derive_input(&input).expect("could not parse args");

    let mut result = quote!();

    if args.columns().any(|col| col.postgres.is_some()) {
        let postgres_impl = args.generate_schema_impl(Database::Postgres);
        result = quote! {
            #result
            #postgres_impl
        }
    }

    if args.columns().any(|col| col.sqlite.is_some()) {
        let sqlite_impl = args.generate_schema_impl(Database::Sqlite);
        result = quote! {
            #result
            #sqlite_impl
        }
    }

    result.into()
}
