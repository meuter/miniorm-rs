use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, DeriveInput, Meta, MetaList};

fn generate_has_table(input: DeriveInput) -> TokenStream {
    let ident = input.ident;
    let table = ident.to_string().to_lowercase();

    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        syn::Data::Enum(_) => panic!("only structs are supported"),
        syn::Data::Union(_) => panic!("only structs are supported"),
    };

    let table_entries = fields.into_iter().map(|field| {
        let field_ident = field.ident.unwrap();
        let field_str = field_ident.to_string();

        let col_type = field
            .attrs
            .into_iter()
            .filter_map(|attr| {
                if let Meta::List(meta_list) = &attr.meta {
                    let MetaList { path, tokens, .. } = meta_list;
                    let is_column = path.segments.iter().any(|seg| seg.ident == "column");
                    if is_column {
                        let col_type = tokens.to_string();
                        Some(col_type)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| panic!("missing `#[column(<type>)]` for `{field_str}`, e.g. `#[column(TEXT NOT NULL)]`"));

        quote! {
            (#field_str, #col_type),
        }
    });

    quote! {
        impl ::miniorm::HasTable for #ident {
            const TABLE: ::miniorm::Table = miniorm::Table(
                #table,
                &[ #(#table_entries)* ],
            );
        }
    }
    .into()
}

fn generate_bind(input: DeriveInput) -> TokenStream {
    let ident = input.ident;

    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        syn::Data::Enum(_) => panic!("only structs are supported"),
        syn::Data::Union(_) => panic!("only structs are supported"),
    };

    let match_arms = fields.into_iter().map(|field| {
        let field_ident = field.ident.unwrap();
        let field_str = field_ident.to_string();

        let is_sqlx_json = field.attrs.into_iter().any(|attr| {
            if let Meta::List(meta_list) = &attr.meta {
                let MetaList { path, tokens, .. } = meta_list;
                let is_sqlx = path.segments.iter().any(|seg| seg.ident == "sqlx");
                let is_json = tokens.to_string() == "json";
                is_sqlx && is_json
            } else {
                false
            }
        });

        if is_sqlx_json {
            quote! {
                #field_str => query.bind(::serde_json::to_value(&self.#field_ident).unwrap()),
            }
        } else {
            quote! {
                #field_str => query.bind(self.#field_ident.clone()),
            }
        }
    });

    quote! {
        impl ::miniorm::Bind for #ident {
            fn bind<'q, O>(
                &self,
                query: ::miniorm::PgQueryAs<'q, O>,
                column_name: ::miniorm::ColunmName
            ) -> ::miniorm::PgQueryAs<'q, O> {
                match column_name {
                    #(#match_arms)*
                    _ => query,
                }
            }

        }
    }
    .into()
}

#[proc_macro_derive(Bind)]
pub fn derive_bind(input: TokenStream) -> TokenStream {
    generate_bind(parse(input).unwrap())
}

#[proc_macro_derive(HasTable, attributes(column))]
pub fn derive_has_table(input: TokenStream) -> TokenStream {
    generate_has_table(parse(input).unwrap())
}
