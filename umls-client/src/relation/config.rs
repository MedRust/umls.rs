use std::collections::HashSet;

use crate::{
    core::cui::Cui,
    search::core::{DisplayName, EmptyString},
};

/// A relationship configuration.
///
/// The object is created using [`Builder`].
///
/// # Example
/// ```
/// use umls_client::relation::config::Config;
/// use umls_client::core::cui::Cui;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let cui: Cui = "C0010200".parse()?;
/// let config = Config::builder(cui.clone()).build();
/// assert_eq!(config.cui(), &cui);
/// assert!(config.labels().is_empty());
/// assert!(config.additional_labels().is_empty());
/// assert!(!config.include_obsolete());
/// assert!(!config.include_suppressible());
/// assert!(config.source_vocabularies().is_empty());
/// # Ok(())
/// # }
/// ```
pub struct Config {
    cui: Cui,
    labels: Vec<DisplayName>,
    additional_labels: Vec<DisplayName>,
    include_obsolete: bool,
    include_suppressible: bool,
    source_vocabularies: Vec<DisplayName>,
}

impl Config {
    /// Create and return a [`Builder`] object for given [`Cui`].
    ///
    /// # Example
    /// ```
    /// use umls_client::relation::config::Config;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cui = "C0010200".parse()?;
    /// let config = Config::builder(cui).build();
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(cui: Cui) -> Builder {
        Builder::new(cui)
    }

    /// Returns a reference of [`Cui`].
    pub fn cui(&self) -> &Cui {
        &self.cui
    }

    /// Returns a reference of relationship labels.
    pub fn labels(&self) -> &[DisplayName] {
        &self.labels
    }

    /// Returns reference of additional relationship labels.
    pub fn additional_labels(&self) -> &[DisplayName] {
        &self.additional_labels
    }

    /// return if the response contains obsolete data.
    pub fn include_obsolete(&self) -> bool {
        self.include_obsolete
    }

    /// Return if the response contains suppressible result.
    pub fn include_suppressible(&self) -> bool {
        self.include_suppressible
    }

    /// Returns reference of source vocabularies.
    pub fn source_vocabularies(&self) -> &[DisplayName] {
        &self.source_vocabularies
    }
}

/// A object builder to create [`Config`] object.
///
/// The structure uses builder pattern with mathod chaining to give each of access.
///
/// # Example
/// ```
/// use umls_client::relation::config::Builder;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let cui = "C0010200".parse()?;
/// let builder = Builder::new(cui);
/// # Ok(())
/// # }
/// ```
pub struct Builder {
    cui: Cui,
    labels: Vec<DisplayName>,
    additional_labels: Vec<DisplayName>,
    include_obsolete: bool,
    include_suppressible: bool,
    source_vocabularies: Vec<DisplayName>,
}

impl Builder {
    /// Create a new instance of [`Builder`] for given CUI.
    ///
    /// The builder is created with empty relationship labels, additional relationship labels,
    /// source vocabularies and excluding the obsolete and is_suppressible concepts.
    pub fn new(cui: Cui) -> Self {
        Self {
            cui,
            labels: Vec::new(),
            additional_labels: Vec::new(),
            include_obsolete: false,
            include_suppressible: false,
            source_vocabularies: Vec::new(),
        }
    }

    /// Sets the relationship labels where response relationship belongs to.
    pub fn labels(mut self, labels: Vec<DisplayName>) -> Self {
        self.labels = labels;
        self
    }

    /// Add the relationship label where relationship response belongs to.
    pub fn try_add_label(
        mut self,
        label: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.labels.push(label.try_into()?);
        Ok(self)
    }

    /// sets the additional relationship labels where relationship response belongs to.
    pub fn additional_labels(mut self, additional_labels: Vec<DisplayName>) -> Self {
        self.additional_labels = additional_labels;
        self
    }

    /// Add the additional relationship label where relationship response belongs to.
    pub fn try_add_additional_label(
        mut self,
        additional_label: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.additional_labels.push(additional_label.try_into()?);
        Ok(self)
    }

    /// set to include obsolete relationships for respective CUI.
    pub fn include_obsolete(mut self) -> Self {
        self.include_obsolete = true;
        self
    }

    /// Include suppressible concept relationships.
    pub fn include_suppressible(mut self) -> Self {
        self.include_suppressible = true;
        self
    }

    /// Sets source vocabularies where relationships are belongs to.
    pub fn source_vocabularies(mut self, source_vocabularies: Vec<DisplayName>) -> Self {
        self.source_vocabularies = source_vocabularies;
        self
    }

    /// Add source vocabulary where relationships are belongs to.
    pub fn try_add_source_vocabulary(
        mut self,
        source_vocabulary: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.source_vocabularies.push(source_vocabulary.try_into()?);
        Ok(self)
    }

    /// Build [`Config`] object.
    pub fn build(self) -> Config {
        let labels = self
            .labels
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let additional_labels = self
            .additional_labels
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let source_vocabularies = self
            .source_vocabularies
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        Config {
            cui: self.cui,
            labels,
            additional_labels,
            include_obsolete: self.include_obsolete,
            include_suppressible: self.include_suppressible,
            source_vocabularies,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn build_default_configuration() -> Result<(), Box<dyn std::error::Error>> {
        let cui = "C0010200".parse()?;

        let config = Config::builder(cui).build();

        assert_eq!(config.cui().as_str(), "C0010200");
        assert!(config.labels().is_empty());
        assert!(config.additional_labels().is_empty());
        assert!(config.source_vocabularies().is_empty());
        assert!(!config.include_obsolete());
        assert!(!config.include_suppressible());

        Ok(())
    }

    #[test]
    fn build_configuration() -> Result<(), Box<dyn std::error::Error>> {
        let cui = "C0010200".parse()?;

        let config = Config::builder(cui)
            .try_add_label("CHD")?
            .try_add_label("CHD")?
            .try_add_additional_label("isa")?
            .try_add_additional_label("isa")?
            .try_add_source_vocabulary("Disease or Syndrome")?
            .try_add_source_vocabulary("Disease or Syndrome")?
            .include_suppressible()
            .include_obsolete()
            .build();

        assert_eq!(config.cui().as_str(), "C0010200");

        assert!(!config.labels().is_empty());
        assert_eq!(config.labels().first().unwrap().as_str(), "CHD");

        assert!(!config.additional_labels().is_empty());
        assert_eq!(config.additional_labels().first().unwrap().as_str(), "isa");

        assert!(!config.source_vocabularies().is_empty());
        assert_eq!(
            config.source_vocabularies().first().unwrap().as_str(),
            "Disease or Syndrome"
        );

        assert!(config.include_obsolete());
        assert!(config.include_suppressible());

        Ok(())
    }
}
