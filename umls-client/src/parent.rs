/// UMLS parent client.
pub struct Parent<C, T> {
    client: C,
    value: T,
}

impl<T> Parent<umls_http::WithoutClient, T> {
    /// Create and return a new instance of [`Parent`] for given value.
    ///
    /// The client is created with http client [`umls_http::WithoutClient`].
    pub fn new(value: T) -> Self {
        Self {
            client: umls_http::WithoutClient,
            value,
        }
    }

    /// Create a new [`Parent`] object with specified client.
    pub fn with_client<C>(self, client: C) -> Parent<C, T> {
        Parent {
            client,
            value: self.value,
        }
    }
}

impl<C, T> Parent<C, T> {
    /// Returns a reference of client object.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Returns a reference of value object.
    pub fn value(&self) -> &T {
        &self.value
    }
}
