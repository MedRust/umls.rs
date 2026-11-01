use std::marker::PhantomData;

use http::uri::PathAndQuery;
use secrecy::ExposeSecret;

use crate::{
    atom,
    config::Config,
    core::{aui::Aui, cui::Cui},
    definition,
};

#[cfg(feature = "reqwest")]
use crate::relation;

#[cfg(feature = "reqwest")]
use crate::parent::SourceConcept;

/// Represents URL construction error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid HTTP scheme.
    #[error("Invalid Scheme")]
    InvalidScheme,

    /// No athority found.
    #[error("Athority not found")]
    NoAthority,

    /// Represents HTTP error.
    #[error("Http Error: {0}")]
    Http(#[source] http::Error),
}

/// A `Result` typedef to use with the [`Error`] type
pub type Result<T> = std::result::Result<T, Error>;

fn construct_uri<T>(
    base_url: impl TryInto<http::Uri, Error = http::uri::InvalidUri>,
    p_and_q: T,
) -> Result<http::Uri>
where
    T: TryInto<PathAndQuery>,
    T::Error: Into<http::Error>,
{
    let base_url = base_url
        .try_into()
        .map_err(|x| x.into())
        .map_err(Error::Http)?;
    let authority = base_url.authority().ok_or(Error::NoAthority)?.as_str();
    let scheme = base_url.scheme_str().ok_or(Error::InvalidScheme)?;
    let base_path = base_url.path().trim_end_matches("/");
    let p_and_q = p_and_q
        .try_into()
        .map_err(|x| x.into())
        .map_err(Error::Http)?;
    let path = format!("{}{}", base_path, p_and_q);

    http::Uri::builder()
        .scheme(scheme)
        .authority(authority)
        .path_and_query(path)
        .build()
        .map_err(Error::Http)
}

/// A umls type.
pub mod types {
    /// A concept url type.
    pub struct Concept;
    /// A atom url type.
    pub struct Atom;
    /// A source url type.
    pub struct Source;

    /// A atoms types.
    pub mod atoms {
        /// A all atom type url.
        pub struct All;
        /// A preferred atom url type.
        pub struct Preferred;
    }
}

/// Represents info url.
#[derive(Debug, Clone)]
pub struct Info<'a, T, V> {
    _type: PhantomData<T>,
    config: &'a Config,
    value: V,
}

impl<'a, T, V> Info<'a, T, V> {
    /// Create [`Info`] object.
    pub fn new(config: &'a Config, value: V) -> Self {
        Self {
            _type: PhantomData,
            config,
            value,
        }
    }
}

impl<'a> Info<'a, types::Concept, Cui> {
    /// Returns the concept info url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/CUI/{}?apiKey={}",
            self.config.version().as_str(),
            self.value.as_str(),
            self.config.api_key().expose_secret()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

impl<'a> Info<'a, types::Atom, Aui> {
    /// Returns atom info url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/AUI/{}?apiKey={}",
            self.config.version(),
            self.value,
            self.config.api_key().expose_secret()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

/// Represents atom urlconstruction builder.
pub struct Atom<'a, T, O, V> {
    _type: PhantomData<T>,
    _option: PhantomData<O>,
    config: &'a Config,
    value: V,
}

impl<'a, T, O, V> Atom<'a, T, O, V> {
    /// Create [`Atom`] object.
    pub fn new(config: &'a Config, value: V) -> Self {
        Atom {
            _type: PhantomData,
            config,
            value,
            _option: PhantomData,
        }
    }
}

impl<'a> Atom<'a, types::Concept, types::atoms::All, Cui> {
    /// Returns concept atoms url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/CUI/{}/atoms?apiKey={}",
            self.config.version(),
            self.value,
            self.config.api_key().expose_secret()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

impl<'a> Atom<'a, types::Concept, types::atoms::Preferred, Cui> {
    /// Returns preferred atom for concept url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/CUI/{}/atoms/preferred?apiKey={}",
            self.config.version(),
            self.value,
            self.config.api_key().expose_secret()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

impl<'a> Atom<'a, types::Concept, types::atoms::All, atom::Batch<atom::Config<Cui>>> {
    /// Returns all atoms for concept url.
    pub fn uri(&self) -> Result<http::Uri> {
        let cui = self.value.value().value();

        let source_vocabularies_params = self
            .value
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let tty_params = self
            .value
            .value()
            .term_types()
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let include_obsolete = self.value.value().is_include_obsolete();
        let include_suppressible = self.value.value().is_include_suppressible();

        let page_number = self.value.number();
        let page_size = self.value.size();
        let language = self
            .value
            .value()
            .language()
            .map(|x| x.as_str())
            .unwrap_or("");

        let p_and_q = format!(
            "/content/{}/CUI/{}/atoms?apiKey={}&sabs={}&ttys={}&language={}&includeObsolete={}&includeSuppressible={}&pageNumber={}&pageSize={}",
            self.config.version(),
            cui,
            self.config.api_key().expose_secret(),
            source_vocabularies_params,
            tty_params,
            language,
            include_obsolete,
            include_suppressible,
            page_number,
            page_size
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

/// Represents definitions url.
pub struct Definition<'a, T, V> {
    _type: PhantomData<T>,
    config: &'a Config,
    value: V,
}

impl<'a, T, V> Definition<'a, T, V> {
    /// Create [`Definition`] object.
    pub fn new(config: &'a Config, value: V) -> Self {
        Self {
            _type: PhantomData,
            config,
            value,
        }
    }

    /// Returns value.
    pub fn value(&self) -> &V {
        &self.value
    }

    /// Returns configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }
}

impl<'a> Definition<'a, types::Concept, Cui> {
    /// Returns concept definitions url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/CUI/{}/definitions?apiKey={}",
            self.config.version().as_str(),
            self.value,
            self.config.api_key().expose_secret()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

impl<'a> Definition<'a, types::Concept, atom::Batch<definition::Config>> {
    /// Returns concept definitions url.
    pub fn uri(&self) -> Result<http::Uri> {
        let sabs = self
            .value
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let p_and_q = format!(
            "/content/{}/CUI/{}/definitions?apiKey={}&sabs={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value.value().cui(),
            self.config.api_key().expose_secret(),
            sabs,
            self.value.size(),
            self.value.number()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

impl<'a> Definition<'a, types::Concept, definition::Config> {
    /// Returns concept definitions url.
    pub fn uri(&self) -> Result<http::Uri> {
        let sabs = self
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let p_and_q = format!(
            "/content/{}/CUI/{}/definitions?apiKey={}&sabs={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value.cui(),
            self.config.api_key().expose_secret(),
            sabs,
            crate::search::core::DEFAULT_PAGE_SIZE,
            crate::search::core::DEFAULT_PAGE_NUMBER
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

/// Represents relations url.
pub struct Relation<'a, T, V> {
    _type: PhantomData<T>,
    config: &'a Config,
    value: V,
}

impl<'a, T, V> Relation<'a, T, V> {
    /// Return new instance of [`Relation`].
    pub fn new(config: &'a Config, value: V) -> Self {
        Self {
            _type: PhantomData,
            config,
            value,
        }
    }

    /// Returns umls configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns value.
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<'a> Relation<'a, types::Concept, Cui> {
    /// Returns relations definitions.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/CUI/{}/relations?apiKey={}",
            self.config.version().as_str(),
            self.value,
            self.config.api_key().expose_secret()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Relation<'a, types::Concept, relation::config::Config> {
    /// Returns concept relations for relation configuration.
    pub fn uri(&self) -> Result<http::Uri> {
        let labels = self
            .value()
            .labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let additional_lables = self
            .value()
            .additional_labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let source_vocabularies = self
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let p_and_q = format!(
            "/content/{}/CUI/{}/relations?apiKey={}&includeRelationLabels={}&includeAdditionalRelationLabels={}&sabs={}&includeObsolete={}&includeSuppressible={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value().cui(),
            self.config.api_key().expose_secret(),
            labels,
            additional_lables,
            source_vocabularies,
            self.value().include_obsolete(),
            self.value().include_suppressible(),
            crate::search::core::DEFAULT_PAGE_SIZE,
            crate::search::core::DEFAULT_PAGE_NUMBER
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Relation<'a, types::Concept, atom::Batch<relation::config::Config>> {
    /// Returns concept relations for configuration with batches.
    pub fn uri(&self) -> Result<http::Uri> {
        let labels = self
            .value()
            .value()
            .labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let additional_lables = self
            .value()
            .value()
            .additional_labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let source_vocabularies = self
            .value()
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let p_and_q = format!(
            "/content/{}/CUI/{}/relations?apiKey={}&includeRelationLabels={}&includeAdditionalRelationLabels={}&sabs={}&includeObsolete={}&includeSuppressible={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value().value().cui(),
            self.config.api_key().expose_secret(),
            labels,
            additional_lables,
            source_vocabularies,
            self.value().value().include_obsolete(),
            self.value().value().include_suppressible(),
            self.value().size(),
            self.value().number()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

/// Represents parent url.
pub struct Parent<'a, T, V> {
    _type: PhantomData<T>,
    config: &'a Config,
    value: V,
}

impl<'a, T, V> Parent<'a, T, V> {
    /// Returns new instance of [`Parent`].
    pub fn new(config: &'a Config, value: V) -> Self {
        Self {
            _type: PhantomData,
            config,
            value,
        }
    }

    /// Returns configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns value.
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<'a> Parent<'a, types::Atom, Aui> {
    /// Returns atom parents url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/AUI/{}/parents?apiKey={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value,
            self.config.api_key().expose_secret(),
            crate::search::core::DEFAULT_PAGE_SIZE,
            crate::search::core::DEFAULT_PAGE_NUMBER
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

impl<'a> Parent<'a, types::Atom, atom::Batch<Aui>> {
    /// Returns atom parents in batches url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/AUI/{}/parents?apiKey={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value().value(),
            self.config.api_key().expose_secret(),
            self.value().size(),
            self.value().number()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Parent<'a, types::Source, SourceConcept> {
    /// Returns a source-asserted parent url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/source/{}/{}/parents?apiKey={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value().source(),
            self.value.code(),
            self.config.api_key().expose_secret(),
            crate::search::core::DEFAULT_PAGE_SIZE,
            crate::search::core::DEFAULT_PAGE_NUMBER
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Parent<'a, types::Source, atom::Batch<SourceConcept>> {
    /// Returns a source-asserted concept in batches url.
    pub fn uri(&self) -> Result<http::Uri> {
        let p_and_q = format!(
            "/content/{}/source/{}/{}/parents?apiKey={}&pageSize={}&pageNumber={}",
            self.config.version().as_str(),
            self.value().value().source(),
            self.value().value().code(),
            self.config.api_key().expose_secret(),
            self.value().size(),
            self.value().number()
        );

        construct_uri(self.config.base_url().as_str(), p_and_q)
    }
}

/// Represents UMLS search url builder.
pub struct Search<'a, V> {
    config: &'a Config,
    value: V,
}

impl<'a, V> Search<'a, V> {
    /// Returns new instalce of [`Search`].
    pub fn new(config: &'a Config, value: V) -> Self {
        Self { config, value }
    }

    /// Returns UMLS configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns a value.
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<'a> Search<'a, crate::search::Request> {
    /// Returns a search url.
    pub fn uri(&self) -> Result<http::Uri> {
        let source_vocabularies_params = self
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let semantic_types_params = self
            .value()
            .semantic_types()
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let semantic_groups_params = self
            .value()
            .semantic_groups()
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let p_and_q = format!(
            "/search/{}?apiKey={}&string={}&inputType={}&includeObsolete={}&includeSuppressible={}&returnIdType={}&sabs={}&searchType={}&semanticTypes={}&semanticGroups={}&partialSearch={}&pageSize={}&pageNumber={}",
            self.config().version(),
            self.config().api_key().expose_secret(),
            self.value().term(),
            self.value().input_type().as_str(),
            self.value().obsolete().is_included(),
            self.value().suppressible().is_included(),
            self.value().return_id_type().as_str(),
            source_vocabularies_params,
            self.value().search_type().as_str(),
            semantic_types_params,
            semantic_groups_params,
            self.value().term_match().allows_partial(),
            self.value().page().size(),
            self.value().page().number(),
        );

        construct_uri(self.config().base_url().as_str(), p_and_q)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        atom, config,
        core::{aui, cui, url::Definition},
        definition,
        search::{
            self,
            core::{DisplayName, InputType, ReturnIdType, SearchType},
        },
    };

    #[cfg(feature = "reqwest")]
    use crate::{parent::SourceConcept, relation};

    use super::{Atom, Error, Info, Parent, Relation, Search, construct_uri, types};

    #[test]
    fn construct_new_uri_from_base_uri() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = "http://localhost:8080/rest";
        let object = construct_uri(base_url, "/new/url")?;
        assert_eq!(object, "http://localhost:8080/rest/new/url");

        let base_url = "http://localhost:8080";
        let object = construct_uri(base_url, "/new/url")?;
        assert_eq!(object, "http://localhost:8080/new/url");

        let base_url = "http://localhost:8080/rest/";
        let object = construct_uri(base_url, "/new/url")?;
        assert_eq!(object, "http://localhost:8080/rest/new/url");

        let base_url = "http://localhost:8080/";
        let object = construct_uri(base_url, "/new/url")?;
        assert_eq!(object, "http://localhost:8080/new/url");

        Ok(())
    }

    #[test]
    fn invalid_scheme_construction_uri() {
        let uri = "http:";
        let result = construct_uri(uri, "");
        assert!(matches!(result, Err(Error::InvalidScheme)));
    }

    #[test]
    fn authority_not_found() {
        let uri = "/path/to/resource";
        let result = construct_uri(uri, "");
        assert!(matches!(result, Err(Error::NoAthority)));
    }

    #[test]
    fn invalid_url() -> Result<(), Box<dyn std::error::Error>> {
        let base_url = "http://localhost:8080";
        let result = construct_uri(base_url, "http://localhost:8080");
        assert!(matches!(result, Err(Error::Http(_))));
        Ok(())
    }

    #[test]
    fn construct_concept_info_uri_for_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Info::<types::Concept, _>::new(&config, cui).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key")
        );
        Ok(())
    }

    #[test]
    fn construct_atom_info_uri_for_aui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let string: DisplayName = "A8345234".parse()?;
        let aui = aui::Aui::new(string)?;

        let uri = Info::<types::Atom, _>::new(&config, aui).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/AUI/A8345234");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_for_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, cui).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_preferred_atom_uri_for_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Atom::<types::Concept, types::atoms::Preferred, _>::new(&config, cui).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(
            uri.path(),
            "/rest/content/current/CUI/C0010200/atoms/preferred"
        );
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_batch_default_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui).build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=&ttys=&language=&includeObsolete=false&includeSuppressible=false&pageNumber=1&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_single_sab() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui)
            .try_add_source_vocabulary("SNOMEDCT_US")?
            .build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=SNOMEDCT_US&ttys=&language=&includeObsolete=false&includeSuppressible=false&pageNumber=1&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_single_multiple_sabs()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui)
            .try_add_source_vocabulary("SNOMEDCT_US")?
            .try_add_source_vocabulary("MSH")?
            .build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=SNOMEDCT_US,MSH&ttys=&language=&includeObsolete=false&includeSuppressible=false&pageNumber=1&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_single_tty() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui).try_add_term_type("PT")?.build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=&ttys=PT&language=&includeObsolete=false&includeSuppressible=false&pageNumber=1&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_multiple_ttys() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui)
            .try_add_term_type("PT")?
            .try_add_term_type("FN")?
            .try_add_term_type("SY")?
            .build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=&ttys=PT,FN,SY&language=&includeObsolete=false&includeSuppressible=false&pageNumber=1&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_include_obsolete() -> Result<(), Box<dyn std::error::Error>>
    {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui)
            .should_include_obsolete(true)
            .build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=&ttys=&language=&includeObsolete=true&includeSuppressible=false&pageNumber=1&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_include_suppressible()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui)
            .should_include_suppressible(true)
            .build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=&ttys=&language=&includeObsolete=false&includeSuppressible=true&pageNumber=1&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_different_page_size()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui)
            .page_size(200.try_into()?)
            .build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=&ttys=&language=&includeObsolete=false&includeSuppressible=false&pageNumber=1&pageSize=200"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_different_page_number()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui)
            .page_number(30.try_into()?)
            .build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&sabs=&ttys=&language=&includeObsolete=false&includeSuppressible=false&pageNumber=30&pageSize=25"
            )
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_uri_using_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Definition::<types::Concept, _>::new(&config, cui).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_default_configuration_in_batches()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();
        let batch = atom::Batch::new(25.try_into()?, 1.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=&pageSize=25&pageNumber=1")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_single_sab_in_batches()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui)
            .try_add_source_vocabulary("SNOMEDCT_US")?
            .build();
        let batch = atom::Batch::new(25.try_into()?, 1.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=SNOMEDCT_US&pageSize=25&pageNumber=1")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_multiple_sabs_in_batches()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui)
            .try_add_source_vocabulary("SNOMEDCT_US")?
            .try_add_source_vocabulary("MSH")?
            .build();
        let batch = atom::Batch::new(25.try_into()?, 1.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=SNOMEDCT_US,MSH&pageSize=25&pageNumber=1")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_page_size() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();
        let batch = atom::Batch::new(200.try_into()?, 1.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=&pageSize=200&pageNumber=1")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_page_number() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();
        let batch = atom::Batch::new(25.try_into()?, 30.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, batch).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=&pageSize=25&pageNumber=30")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_default_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();

        let uri = Definition::<types::Concept, _>::new(&config, def_config).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=&pageSize=25&pageNumber=1")
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_single_sab() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui)
            .try_add_source_vocabulary("SNOMEDCT_US")?
            .build();

        let uri = Definition::<types::Concept, _>::new(&config, def_config).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=SNOMEDCT_US&pageSize=25&pageNumber=1")
        );

        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_multiple_sabs() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui)
            .try_add_source_vocabulary("SNOMEDCT_US")?
            .try_add_source_vocabulary("MSH")?
            .try_add_source_vocabulary("MTH")?
            .build();

        let uri = Definition::<types::Concept, _>::new(&config, def_config).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&sabs=SNOMEDCT_US,MSH,MTH&pageSize=25&pageNumber=1")
        );

        Ok(())
    }

    #[test]
    fn construct_concept_relation_uri_using_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Relation::<types::Concept, _>::new(&config, cui).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key")
        );
        Ok(())
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn construct_concept_relation_uri_with_default_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let relation_config = relation::config::Config::builder(cui).build();

        let uri = Relation::<types::Concept, _>::new(&config, relation_config).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some(
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1"
            )
        );
        Ok(())
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn construct_concept_relation_uri() -> Result<(), Box<dyn std::error::Error>> {
        let relation_config = [
            (
                relation::config::Config::builder("C0010200".parse()?).build(),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .try_add_label("CHD")?
                    .build(),
                "apiKey=some_api_key&includeRelationLabels=CHD&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .include_obsolete()
                    .build(),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=true&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .include_suppressible()
                    .build(),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=true&pageSize=25&pageNumber=1",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .try_add_additional_label("isa")?
                    .build(),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=isa&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .try_add_source_vocabulary("Findings")?
                    .build(),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=Findings&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
        ];

        for (config, query_params) in relation_config {
            let umls_config = config::Config::builder("some_api_key").build()?;

            let uri = Relation::<types::Concept, _>::new(&umls_config, config).uri()?;
            assert_eq!(uri.scheme_str(), Some("https"));
            assert_eq!(
                uri.authority().map(|x| x.as_str()),
                Some("uts-ws.nlm.nih.gov")
            );
            assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
            assert_eq!(
                uri.path_and_query().and_then(|x| x.query()),
                Some(query_params)
            );
        }

        let batch_relation_config = [
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?).build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .try_add_label("CHD")?
                        .build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=CHD&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .include_obsolete()
                        .build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=true&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .include_suppressible()
                        .build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=true&pageSize=25&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .try_add_additional_label("isa")?
                        .build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=isa&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .try_add_source_vocabulary("Findings")?
                        .build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=Findings&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    200.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?).build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=200&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    30.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?).build(),
                ),
                "apiKey=some_api_key&includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=30",
            ),
        ];

        for (config, query_params) in batch_relation_config {
            let umls_config = config::Config::builder("some_api_key").build()?;

            let uri = Relation::<types::Concept, _>::new(&umls_config, config).uri()?;
            assert_eq!(uri.scheme_str(), Some("https"));
            assert_eq!(
                uri.authority().map(|x| x.as_str()),
                Some("uts-ws.nlm.nih.gov")
            );
            assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
            assert_eq!(
                uri.path_and_query().and_then(|x| x.query()),
                Some(query_params)
            );
        }

        Ok(())
    }

    #[test]
    fn construct_atom_parent_uri() -> Result<(), Box<dyn std::error::Error>> {
        let umls_config = config::Config::builder("some_api_key").build()?;
        let string: DisplayName = "A8345234".parse()?;
        let aui = aui::Aui::new(string)?;

        let uri = Parent::<types::Atom, _>::new(&umls_config, aui).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(uri.path(), "/rest/content/current/AUI/A8345234/parents");
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&pageSize=25&pageNumber=1")
        );

        let batch_parents = [
            (
                atom::Batch::new(300.try_into()?, 1.try_into()?, {
                    let display_name: DisplayName = "A8345234".parse()?;
                    aui::Aui::new(display_name)
                }?),
                "apiKey=some_api_key&pageSize=300&pageNumber=1",
            ),
            (
                atom::Batch::new(25.try_into()?, 30.try_into()?, {
                    let display_name: DisplayName = "A8345234".parse()?;
                    aui::Aui::new(display_name)
                }?),
                "apiKey=some_api_key&pageSize=25&pageNumber=30",
            ),
        ];

        for (config, query_params) in batch_parents {
            let uri = Parent::<types::Atom, _>::new(&umls_config, config).uri()?;
            assert_eq!(uri.scheme_str(), Some("https"));
            assert_eq!(
                uri.authority().map(|x| x.as_str()),
                Some("uts-ws.nlm.nih.gov")
            );
            assert_eq!(uri.path(), "/rest/content/current/AUI/A8345234/parents");
            assert_eq!(
                uri.path_and_query().and_then(|x| x.query()),
                Some(query_params)
            );
        }

        Ok(())
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn construct_source_concept_parent_uri() -> Result<(), Box<dyn std::error::Error>> {
        let umls_config = config::Config::builder("some_api_key").build()?;
        let source_concept = SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?);

        let uri = Parent::<types::Source, _>::new(&umls_config, source_concept).uri()?;
        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(
            uri.authority().map(|x| x.as_str()),
            Some("uts-ws.nlm.nih.gov")
        );
        assert_eq!(
            uri.path(),
            "/rest/content/current/source/SNOMEDCT_US/9468002/parents"
        );
        assert_eq!(
            uri.path_and_query().and_then(|x| x.query()),
            Some("apiKey=some_api_key&pageSize=25&pageNumber=1")
        );

        let sets = [
            (
                atom::Batch::new(
                    300.try_into()?,
                    1.try_into()?,
                    SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?),
                ),
                "apiKey=some_api_key&pageSize=300&pageNumber=1",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    30.try_into()?,
                    SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?),
                ),
                "apiKey=some_api_key&pageSize=25&pageNumber=30",
            ),
            (
                atom::Batch::new(
                    300.try_into()?,
                    30.try_into()?,
                    SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?),
                ),
                "apiKey=some_api_key&pageSize=300&pageNumber=30",
            ),
        ];

        for (batch, query) in sets {
            let uri = Parent::<types::Source, _>::new(&umls_config, batch).uri()?;
            assert_eq!(uri.scheme_str(), Some("https"));
            assert_eq!(
                uri.authority().map(|x| x.as_str()),
                Some("uts-ws.nlm.nih.gov")
            );
            assert_eq!(
                uri.path(),
                "/rest/content/current/source/SNOMEDCT_US/9468002/parents"
            );
            assert_eq!(uri.path_and_query().and_then(|x| x.query()), Some(query));
        }

        Ok(())
    }

    #[test]
    fn construct_search_uri() -> Result<(), Box<dyn std::error::Error>> {
        let sets = [
            (
                search::Request::builder("Cough".try_into()?).build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .obsolete(search::core::Inclusion::Include)
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=true&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .suppressible(search::core::Inclusion::Include)
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=true&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .input_type(InputType::Code)
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=code&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .return_id_type(ReturnIdType::Aui)
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=aui&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .try_add_source_vocabulary("SNOMEDCT_US")?
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=SNOMEDCT_US&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .try_add_source_vocabulary("SNOMEDCT_US")?
                    .try_add_source_vocabulary("MSH")?
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=SNOMEDCT_US,MSH&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .search_type(SearchType::Exact)
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=exact&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .try_add_semantic_type("Findings")?
                    .build(),
                "apiKey=some_api_key&string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=Findings&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1",
            ),
        ];

        for (request, query) in sets {
            let umls_config = config::Config::builder("some_api_key").build()?;

            let uri = Search::new(&umls_config, request).uri()?;
            assert_eq!(uri.scheme_str(), Some("https"));
            assert_eq!(
                uri.authority().map(|x| x.as_str()),
                Some("uts-ws.nlm.nih.gov")
            );
            assert_eq!(uri.path(), "/rest/search/current");
            assert_eq!(uri.path_and_query().and_then(|x| x.query()), Some(query));
        }

        Ok(())
    }
}
