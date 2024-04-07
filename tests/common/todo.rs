use miniorm::Entity;
use sqlx::FromRow;

#[derive(Debug, Clone, Eq, PartialEq, FromRow, Entity, Hash)]
#[cfg_attr(feature = "axum", derive(serde::Serialize, serde::Deserialize))]
pub struct Todo {
    #[column(TEXT NOT NULL)]
    description: String,

    #[column(BOOLEAN NOT NULL DEFAULT false)]
    done: bool,
}

impl Todo {
    pub fn new(description: impl AsRef<str>) -> Self {
        let description = description.as_ref().to_string();
        let done = false;
        Todo { description, done }
    }

    #[allow(unused)]
    pub fn is_done(&self) -> bool {
        self.done
    }

    #[allow(unused)]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[allow(unused)]
    pub fn mark_as_done(&mut self) {
        self.done = true;
    }
}
