use sqlx::{ColumnIndex, Decode, FromRow, Row, Type};
use std::ops::{Deref, DerefMut};

/// [`WithId`] is a wrapper of struct that provides an
/// additional id field which is used in the database
/// to identify the entity.
///
/// [`WithId`] provides some blanket implementation
/// of common traits provided that the underlying
/// type implements these traits:
/// - [`Debug`] and [`Clone`]
/// - [`Eq`] and [`PartialEq`]
pub struct WithId<E> {
    /// the wrapped entity
    inner: E,
    /// the `id` used to identify the `inner` entity
    id: i64,
}

impl<E> WithId<E> {
    /// Creates a wrapped entity with the provided id.
    ///
    /// # Arguments
    /// - `inner`: the entity to be wrapped
    /// - `id`: the id
    ///
    /// # Example
    ///
    /// ```
    /// let with_id = miniorm::WithId::new("miniorm", 1);
    /// assert_eq!(with_id.id(), 1);
    /// assert_eq!(with_id.into_inner(), "miniorm");
    /// ```
    pub fn new(inner: E, id: i64) -> Self {
        WithId { inner, id }
    }

    /// Extracts the inner entity
    ///
    /// # Example
    ///
    /// ```
    /// let with_id = miniorm::WithId::new("miniorm", 10);
    /// assert_eq!(with_id.into_inner(), "miniorm");
    ///
    /// ```
    pub fn into_inner(self) -> E {
        self.inner
    }

    /// Returns a reference to inner entity
    /// # Example
    ///
    /// ```
    /// let with_id = miniorm::WithId::new("miniorm", 10);
    /// assert_eq!(with_id.inner(), &"miniorm");
    ///
    /// ```
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Returns a mutable reference to the inner entity
    ///
    /// # Example
    ///
    /// ```
    /// let mut with_id = miniorm::WithId::new("miniorm", 10);
    /// *with_id.inner_mut() = "miniorm indeed!";
    /// assert_eq!(with_id.inner(), &"miniorm indeed!");
    ///
    /// ```
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Extracts the id and discards the entity
    ///
    /// # Example
    ///
    /// ```
    /// let with_id = miniorm::WithId::new("miniorm", 10);
    /// assert_eq!(with_id.id(), 10);
    ///
    /// ```
    pub fn id(&self) -> i64 {
        self.id
    }
}

impl<E> Deref for WithId<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E> DerefMut for WithId<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, R: Row, E> FromRow<'a, R> for WithId<E>
where
    R: Row,
    E: FromRow<'a, R>,
    &'a str: ColumnIndex<R>,
    i64: Decode<'a, R::Database>,
    i64: Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let inner = E::from_row(row)?;
        let id = row.try_get("id")?;
        Ok(Self { inner, id })
    }
}

impl<E: Clone> Clone for WithId<E> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl<E: PartialEq> PartialEq for WithId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.inner == other.inner
    }
}

impl<E: Eq> Eq for WithId<E> {}

impl<E: std::fmt::Debug> std::fmt::Debug for WithId<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WithId").field("id", &self.id).field("inner", &self.inner).finish()
    }
}

#[cfg(feature = "serde")]
mod with_serde {
    use super::WithId;
    use serde::{
        de::{self, MapAccess, SeqAccess, Visitor},
        Deserialize,
    };
    use std::marker::PhantomData;

    #[derive(Deserialize)]
    #[serde(field_identifier, rename_all = "lowercase")]
    pub(crate) enum WithIdFields {
        Inner,
        Id,
    }

    pub(crate) struct WithIdVisitor<E> {
        inner: PhantomData<E>,
    }

    impl<E> WithIdVisitor<E> {
        pub(crate) fn new() -> Self {
            WithIdVisitor { inner: PhantomData }
        }
    }

    impl<'de, E: Deserialize<'de>> Visitor<'de> for WithIdVisitor<E> {
        type Value = WithId<E>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("struct WithId")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let id = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let inner = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
            Ok(Self::Value::new(inner, id))
        }

        fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut inner = None;
            let mut id = None;
            while let Some(key) = map.next_key()? {
                match key {
                    WithIdFields::Inner => {
                        if inner.is_some() {
                            return Err(de::Error::duplicate_field("inner"));
                        }
                        inner = Some(map.next_value()?);
                    }
                    WithIdFields::Id => {
                        if id.is_some() {
                            return Err(de::Error::duplicate_field("nanos"));
                        }
                        id = Some(map.next_value()?);
                    }
                }
            }
            let inner = inner.ok_or_else(|| de::Error::missing_field("inner"))?;
            let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
            Ok(Self::Value::new(inner, id))
        }
    }
}

#[cfg(feature = "serde")]
impl<E: serde::Serialize> serde::Serialize for WithId<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("WithId", 2)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("inner", &self.inner)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, E: serde::Deserialize<'de>> serde::Deserialize<'de> for WithId<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["inner", "id"];
        deserializer.deserialize_struct("WithId", FIELDS, with_serde::WithIdVisitor::<E>::new())
    }
}

#[cfg(test)]
mod test {
    use crate::WithId;

    #[test]
    fn is_clone_if_inner_is_clone() {
        #[derive(Clone)]
        struct Foo;

        let foo = WithId::new(Foo, 10);
        let _ = foo.clone();
    }

    #[test]
    fn is_eq_if_inner_is_eq_and_debug_if_inner_is_debug() {
        #[derive(Eq, PartialEq, Debug)]
        struct Foo(u8);

        let left = WithId::new(Foo(1), 10);
        let right = WithId::new(Foo(1), 10);
        assert_eq!(left, right);
        let right = WithId::new(Foo(1), 11);
        assert_ne!(left, right);
        let right = WithId::new(Foo(2), 10);
        assert_ne!(left, right);
    }

    #[cfg(feature = "serde")]
    mod serde {
        use crate::WithId;
        use serde::{Deserialize, Serialize};

        #[test]
        fn is_serialize_if_inner_is_serialize() {
            #[derive(Serialize)]
            struct Foo {
                x: u32,
            }

            let with_id = WithId::new(Foo { x: 420 }, 69);
            assert_eq!(serde_json::to_string(&with_id).unwrap(), r#"{"id":69,"inner":{"x":420}}"#);
        }

        #[test]
        fn is_deserialize_if_inner_is_deserialize() {
            #[derive(Deserialize)]
            struct Foo(u32);

            let with_id: WithId<Foo> = serde_json::from_str(r#"{"id":69,"inner":420}"#).unwrap();
            assert_eq!(with_id.inner.0, 420);
            assert_eq!(with_id.id, 69);
        }
    }
}
