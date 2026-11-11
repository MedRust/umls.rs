use serde::Deserialize;

/// An owned collection of converted results.
#[derive(Debug)]
pub struct Content<T> {
    pub(crate) list: Vec<T>,
}

impl<T> Content<T> {
    /// Creates content from an owned result list.
    pub(crate) fn new(list: Vec<T>) -> Self {
        Self { list }
    }

    /// Returns the results as a slice in API-provided order.
    pub fn list(&self) -> &[T] {
        &self.list
    }
}

impl<T> AsRef<[T]> for Content<T> {
    fn as_ref(&self) -> &[T] {
        &self.list
    }
}

impl<U> From<Vec<U>> for Content<U> {
    fn from(value: Vec<U>) -> Self {
        Content::new(value)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Content<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let list = Vec::<T>::deserialize(deserializer)?;

        Ok(Content::new(list))
    }
}
