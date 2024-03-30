use darling::FromField;
use syn::{Field, Ident, Meta};

#[derive(Clone, Debug, FromField)]
#[darling(attributes(miniorm, sqlx))]
struct InnerColumn {
    ident: Option<Ident>,
    postgres: Option<String>,
    sqlite: Option<String>,
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
                } else if list.path.is_ident("column") {
                    col.sqlite = Some(schema.clone());
                    col.postgres = Some(schema);
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

    pub fn json(&self) -> bool {
        self.0.json
    }

    pub fn skip(&self) -> bool {
        self.0.skip
    }
}
