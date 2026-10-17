use umls_http::WithoutClient;

/// A UMLS concept relation client.
pub struct Relation<C, V> {
    client: C,
    value: V,
}

impl<C, V> Relation<C, V> {
    /// Return a reference of client.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Return a reference of value.
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<V> Relation<WithoutClient, V> {
    /// Create a new [`Relation`] client for given value.
    ///
    /// The [`Relation`] client is created suing [`WithoutClient`].
    pub fn new(value: V) -> Self {
        Self {
            client: WithoutClient,
            value,
        }
    }

    /// Create a [`Relation`] object with different client.
    pub fn with_client<C>(self, client: C) -> Relation<C, V> {
        Relation {
            client,
            value: self.value,
        }
    }
}
