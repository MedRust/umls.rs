use std::marker::PhantomData;

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

    /// Returns UMLS configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns value.
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<'a> Info<'a, types::Concept, Cui> {
    /// Returns the concept info url.
    pub fn url(&self) -> url::Url {
        let version = self.config().version().as_str().to_string();
        let mut url = self.config().base_url().clone();

        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(self.value.as_str());

        self.config().authenticated_url(url)
    }
}

impl<'a> Info<'a, types::Atom, &Aui> {
    /// Returns atom info url.
    pub fn url(&self) -> url::Url {
        let version = self.config().version().as_str().to_string();
        let aui = self.value.as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("AUI")
            .push(aui);

        self.config().authenticated_url(url)
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

    /// Returns UMLS configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns value.
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<'a> Atom<'a, types::Concept, types::atoms::All, Cui> {
    /// Returns concept atoms url.
    pub fn url(&self) -> url::Url {
        let version = self.config().version().as_str().to_string();
        let cui = self.value().as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("atoms");

        self.config().authenticated_url(url)
    }
}

impl<'a> Atom<'a, types::Concept, types::atoms::Preferred, Cui> {
    /// Returns preferred atom for concept url.
    pub fn url(&self) -> url::Url {
        let version = self.config().version().as_str().to_string();
        let cui = self.value().as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("atoms")
            .push("preferred");

        self.config().authenticated_url(url)
    }
}

impl<'a> Atom<'a, types::Concept, types::atoms::All, &atom::Batch<atom::Config<Cui>>> {
    /// Returns all atoms for concept url.
    pub fn url(&self) -> url::Url {
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

        let version = self.config().version().as_str().to_string();
        let cui = self.value().value().value().as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("atoms");

        url.query_pairs_mut()
            .clear()
            .append_pair("sabs", &source_vocabularies_params)
            .append_pair("ttys", &tty_params)
            .append_pair("language", language)
            .append_pair("includeObsolete", &include_obsolete.to_string())
            .append_pair("includeSuppressible", &include_suppressible.to_string())
            .append_pair("pageNumber", &page_number.to_string())
            .append_pair("pageSize", &page_size.to_string())
            .finish();

        self.config().authenticated_url(url)
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
    pub fn url(&self) -> url::Url {
        let version = self.config().version().to_string();
        let cui = self.value().as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("definitions");

        self.config().authenticated_url(url)
    }
}

impl<'a> Definition<'a, types::Concept, &atom::Batch<definition::Config>> {
    /// Returns concept definitions url.
    pub fn url(&self) -> url::Url {
        let sabs = self
            .value
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let version = self.config().version().to_string();
        let cui = self.value().value().cui().as_str();
        let page_size = self.value().size().to_string();
        let page_number = self.value().number().to_string();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("definitions");

        url.query_pairs_mut()
            .append_pair("sabs", &sabs)
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
    }
}

impl<'a> Definition<'a, types::Concept, &definition::Config> {
    /// Returns concept definitions url.
    pub fn url(&self) -> url::Url {
        let sabs = self
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let version = self.config().version().to_string();
        let cui = self.value().cui().as_str();
        let page_size = crate::search::core::DEFAULT_PAGE_SIZE.to_string();
        let page_number = crate::search::core::DEFAULT_PAGE_NUMBER.to_string();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("definitions");

        url.query_pairs_mut()
            .append_pair("sabs", &sabs)
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
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
    pub fn url(&self) -> url::Url {
        let version = self.config().version().to_string();
        let cui = self.value().as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("relations");

        self.config().authenticated_url(url)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Relation<'a, types::Concept, &relation::config::Config> {
    /// Returns concept relations for relation configuration.
    pub fn url(&self) -> url::Url {
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

        let include_obsolete = self.value().include_obsolete().to_string();
        let include_suppressible = self.value().include_suppressible().to_string();
        let page_size = crate::search::core::DEFAULT_PAGE_SIZE.to_string();
        let page_number = crate::search::core::DEFAULT_PAGE_NUMBER.to_string();

        let version = self.config().version().to_string();
        let cui = self.value().cui().as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("relations");

        url.query_pairs_mut()
            .append_pair("includeRelationLabels", &labels)
            .append_pair("includeAdditionalRelationLabels", &additional_lables)
            .append_pair("sabs", &source_vocabularies)
            .append_pair("includeObsolete", &include_obsolete)
            .append_pair("includeSuppressible", &include_suppressible)
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Relation<'a, types::Concept, &atom::Batch<relation::config::Config>> {
    /// Returns concept relations for configuration with batches.
    pub fn url(&self) -> url::Url {
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

        let include_obsolete = self.value().value().include_obsolete().to_string();
        let include_suppressible = self.value().value().include_suppressible().to_string();
        let page_size = self.value().size().to_string();
        let page_number = self.value().number().to_string();

        let version = self.config().version().to_string();
        let cui = self.value().value().cui().as_str();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("CUI")
            .push(cui)
            .push("relations");

        url.query_pairs_mut()
            .append_pair("includeRelationLabels", &labels)
            .append_pair("includeAdditionalRelationLabels", &additional_lables)
            .append_pair("sabs", &source_vocabularies)
            .append_pair("includeObsolete", &include_obsolete)
            .append_pair("includeSuppressible", &include_suppressible)
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
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
    pub fn url(&self) -> url::Url {
        let version = self.config().version().to_string();
        let aui = self.value().as_str();
        let page_size = crate::search::core::DEFAULT_PAGE_SIZE.to_string();
        let page_number = crate::search::core::DEFAULT_PAGE_NUMBER.to_string();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("AUI")
            .push(aui)
            .push("parents");

        url.query_pairs_mut()
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
    }
}

impl<'a> Parent<'a, types::Atom, &atom::Batch<Aui>> {
    /// Returns atom parents in batches url.
    pub fn url(&self) -> url::Url {
        let version = self.config().version().to_string();
        let aui = self.value().value().as_str();
        let page_size = self.value().size().to_string();
        let page_number = self.value().number().to_string();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("AUI")
            .push(aui)
            .push("parents");

        url.query_pairs_mut()
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Parent<'a, types::Source, SourceConcept> {
    /// Returns a source-asserted parent url.
    pub fn url(&self) -> url::Url {
        let version = self.config().version().to_string();
        let source = self.value().source().as_str();
        let code = self.value().code().as_str();
        let page_number = crate::search::core::DEFAULT_PAGE_NUMBER.to_string();
        let page_size = crate::search::core::DEFAULT_PAGE_SIZE.to_string();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("source")
            .push(source)
            .push(code)
            .push("parents");

        url.query_pairs_mut()
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Parent<'a, types::Source, &atom::Batch<SourceConcept>> {
    /// Returns a source-asserted concept in batches url.
    pub fn url(&self) -> url::Url {
        let version = self.config().version().to_string();
        let source = self.value().value().source().as_str();
        let code = self.value().value().code().as_str();
        let page_number = self.value().number().to_string();
        let page_size = self.value().size().to_string();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("content")
            .push(&version)
            .push("source")
            .push(source)
            .push(code)
            .push("parents");

        url.query_pairs_mut()
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
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

impl<'a> Search<'a, &crate::search::Request> {
    /// Returns a search url.
    pub fn url(&self) -> url::Url {
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

        let version = self.config().version().to_string();
        let term = self.value().term().as_str();
        let input_type = self.value().input_type().as_str().to_string();
        let include_obsolete = self.value().obsolete().is_included().to_string();
        let include_suppressible = self.value().suppressible().is_included().to_string();
        let return_id_type = self.value().return_id_type().as_str();
        let search_type = self.value().search_type().as_str();
        let partial_search = self.value().term_match().allows_partial().to_string();
        let page_number = self.value().page().number().to_string();
        let page_size = self.value().page().size().to_string();

        let mut url = self.config().base_url().clone();
        url.path_segments_mut()
            .expect("The base url should be a base.")
            .pop_if_empty()
            .push("search")
            .push(&version);

        url.query_pairs_mut()
            .append_pair("string", term)
            .append_pair("inputType", &input_type)
            .append_pair("includeObsolete", &include_obsolete)
            .append_pair("includeSuppressible", &include_suppressible)
            .append_pair("returnIdType", return_id_type)
            .append_pair("sabs", &source_vocabularies_params)
            .append_pair("searchType", search_type)
            .append_pair("semanticTypes", &semantic_types_params)
            .append_pair("semanticGroups", &semantic_groups_params)
            .append_pair("partialSearch", &partial_search)
            .append_pair("pageSize", &page_size)
            .append_pair("pageNumber", &page_number)
            .finish();

        self.config().authenticated_url(url)
    }
}

#[cfg(test)]
mod tests {

    use std::borrow::Cow;

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

    use super::{Atom, Info, Parent, Relation, Search, types};

    #[test]
    fn construct_concept_info_uri_for_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Info::<types::Concept, _>::new(&config, cui).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200");
        assert_eq!(uri.query(), Some("apiKey=some_api_key"));
        Ok(())
    }

    #[test]
    fn construct_atom_info_uri_for_aui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let string: DisplayName = "A8345234".parse()?;
        let aui = aui::Aui::new(string)?;

        let uri = Info::<types::Atom, _>::new(&config, &aui).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/AUI/A8345234");
        assert_eq!(uri.query(), Some("apiKey=some_api_key"));
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_for_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, cui).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_eq!(uri.query(), Some("apiKey=some_api_key"));
        Ok(())
    }

    #[test]
    fn construct_concept_preferred_atom_uri_for_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Atom::<types::Concept, types::atoms::Preferred, _>::new(&config, cui).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(
            uri.path(),
            "/rest/content/current/CUI/C0010200/atoms/preferred"
        );
        assert_eq!(uri.query(), Some("apiKey=some_api_key"));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn assert_concept_atoms_query(
        url: &url::Url,
        sabs: &str,
        ttys: &str,
        language: &str,
        include_obsolete: &str,
        include_suppresible: &str,
        page_number: &str,
        page_size: &str,
        api_key: &str,
    ) {
        let mut query = url.query_pairs();
        assert_eq!(
            query.count(),
            8,
            "The concept atom query should contain 8 query parameters"
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("sabs"), Cow::Borrowed(sabs)))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("ttys"), Cow::Borrowed(ttys)))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("language"), Cow::Borrowed(language)))
        );
        assert_eq!(
            query.next(),
            Some((
                Cow::Borrowed("includeObsolete"),
                Cow::Borrowed(include_obsolete)
            ))
        );
        assert_eq!(
            query.next(),
            Some((
                Cow::Borrowed("includeSuppressible"),
                Cow::Borrowed(include_suppresible)
            ))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("pageNumber"), Cow::Borrowed(page_number)))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("pageSize"), Cow::Borrowed(page_size)))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("apiKey"), Cow::Borrowed(api_key)))
        );
    }

    #[test]
    fn construct_concept_atoms_uri_with_batch_default_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui).build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");
        assert_concept_atoms_query(
            &uri,
            "",
            "",
            "",
            "false",
            "false",
            "1",
            "25",
            "some_api_key",
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

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(
            &uri,
            "SNOMEDCT_US",
            "",
            "",
            "false",
            "false",
            "1",
            "25",
            "some_api_key",
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

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(
            &uri,
            "SNOMEDCT_US,MSH",
            "",
            "",
            "false",
            "false",
            "1",
            "25",
            "some_api_key",
        );
        Ok(())
    }

    #[test]
    fn construct_concept_atoms_uri_with_single_tty() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let batch = atom::Config::builder(cui).try_add_term_type("PT")?.build();

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(
            &uri,
            "",
            "PT",
            "",
            "false",
            "false",
            "1",
            "25",
            "some_api_key",
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

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(
            &uri,
            "",
            "PT,FN,SY",
            "",
            "false",
            "false",
            "1",
            "25",
            "some_api_key",
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

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(&uri, "", "", "", "true", "false", "1", "25", "some_api_key");
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

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(&uri, "", "", "", "false", "true", "1", "25", "some_api_key");
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

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(
            &uri,
            "",
            "",
            "",
            "false",
            "false",
            "1",
            "200",
            "some_api_key",
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

        let uri = Atom::<types::Concept, types::atoms::All, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/atoms");

        assert_concept_atoms_query(
            &uri,
            "",
            "",
            "",
            "false",
            "false",
            "30",
            "25",
            "some_api_key",
        );
        Ok(())
    }

    #[test]
    fn construct_concept_definition_uri_using_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Definition::<types::Concept, _>::new(&config, cui).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_eq!(uri.query(), Some("apiKey=some_api_key"));
        Ok(())
    }

    fn assert_concept_definition(
        url: &url::Url,
        sabs: &str,
        page_size: &str,
        page_number: &str,
        api_key: &str,
    ) {
        let mut query = url.query_pairs();
        assert_eq!(query.count(), 4);
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("sabs"), Cow::Borrowed(sabs)))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("pageSize"), Cow::Borrowed(page_size)))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("pageNumber"), Cow::Borrowed(page_number)))
        );
        assert_eq!(
            query.next(),
            Some((Cow::Borrowed("apiKey"), Cow::Borrowed(api_key)))
        );
    }

    #[test]
    fn construct_concept_definition_with_default_configuration_in_batches()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();
        let batch = atom::Batch::new(25.try_into()?, 1.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_concept_definition(&uri, "", "25", "1", "some_api_key");
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

        let uri = Definition::<types::Concept, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");

        assert_concept_definition(&uri, "SNOMEDCT_US", "25", "1", "some_api_key");
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

        let uri = Definition::<types::Concept, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_concept_definition(&uri, "SNOMEDCT_US,MSH", "25", "1", "some_api_key");
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_page_size() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();
        let batch = atom::Batch::new(200.try_into()?, 1.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_concept_definition(&uri, "", "200", "1", "some_api_key");
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_page_number() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();
        let batch = atom::Batch::new(25.try_into()?, 30.try_into()?, def_config);

        let uri = Definition::<types::Concept, _>::new(&config, &batch).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_concept_definition(&uri, "", "25", "30", "some_api_key");
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_default_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui).build();

        let uri = Definition::<types::Concept, _>::new(&config, &def_config).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_concept_definition(&uri, "", "25", "1", "some_api_key");
        Ok(())
    }

    #[test]
    fn construct_concept_definition_with_single_sab() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let def_config = definition::Config::builder(cui)
            .try_add_source_vocabulary("SNOMEDCT_US")?
            .build();

        let uri = Definition::<types::Concept, _>::new(&config, &def_config).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_concept_definition(&uri, "SNOMEDCT_US", "25", "1", "some_api_key");

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

        let uri = Definition::<types::Concept, _>::new(&config, &def_config).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/definitions");
        assert_concept_definition(&uri, "SNOMEDCT_US,MSH,MTH", "25", "1", "some_api_key");

        Ok(())
    }

    #[test]
    fn construct_concept_relation_uri_using_cui() -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;

        let uri = Relation::<types::Concept, _>::new(&config, cui).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
        assert_eq!(uri.query(), Some("apiKey=some_api_key"));
        Ok(())
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn construct_concept_relation_uri_with_default_configuration()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = config::Config::builder("some_api_key").build()?;
        let cui: cui::Cui = "C0010200".parse()?;
        let relation_config = relation::config::Config::builder(cui).build();

        let uri = Relation::<types::Concept, _>::new(&config, &relation_config).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
        assert_eq!(
            uri.query(),
            Some(
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key"
            )
        );
        Ok(())
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn construct_concept_relation_url() -> Result<(), Box<dyn std::error::Error>> {
        let relation_config = [
            (
                relation::config::Config::builder("C0010200".parse()?).build(),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .try_add_label("CHD")?
                    .build(),
                "includeRelationLabels=CHD&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .include_obsolete()
                    .build(),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=true&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .include_suppressible()
                    .build(),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=true&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .try_add_additional_label("isa")?
                    .build(),
                "includeRelationLabels=&includeAdditionalRelationLabels=isa&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                relation::config::Config::builder("C0010200".parse()?)
                    .try_add_source_vocabulary("Findings")?
                    .build(),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=Findings&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
        ];

        for (config, query_params) in relation_config {
            let umls_config = config::Config::builder("some_api_key").build()?;

            let uri = Relation::<types::Concept, _>::new(&umls_config, &config).url();
            assert_eq!(uri.scheme(), "https");
            assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
            assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
            assert_eq!(uri.query(), Some(query_params));
        }

        let batch_relation_config = [
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?).build(),
                ),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .try_add_label("CHD")?
                        .build(),
                ),
                "includeRelationLabels=CHD&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .include_obsolete()
                        .build(),
                ),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=true&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .include_suppressible()
                        .build(),
                ),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=true&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .try_add_additional_label("isa")?
                        .build(),
                ),
                "includeRelationLabels=&includeAdditionalRelationLabels=isa&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?)
                        .try_add_source_vocabulary("Findings")?
                        .build(),
                ),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=Findings&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    200.try_into()?,
                    1.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?).build(),
                ),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=200&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    30.try_into()?,
                    relation::config::Config::builder("C0010200".parse()?).build(),
                ),
                "includeRelationLabels=&includeAdditionalRelationLabels=&sabs=&includeObsolete=false&includeSuppressible=false&pageSize=25&pageNumber=30&apiKey=some_api_key",
            ),
        ];

        for (config, query_params) in batch_relation_config {
            let umls_config = config::Config::builder("some_api_key").build()?;

            let uri = Relation::<types::Concept, _>::new(&umls_config, &config).url();
            assert_eq!(uri.scheme(), "https");
            assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
            assert_eq!(uri.path(), "/rest/content/current/CUI/C0010200/relations");
            assert_eq!(uri.query(), Some(query_params));
        }

        Ok(())
    }

    #[test]
    fn construct_atom_parent_url() -> Result<(), Box<dyn std::error::Error>> {
        let umls_config = config::Config::builder("some_api_key").build()?;
        let string: DisplayName = "A8345234".parse()?;
        let aui = aui::Aui::new(string)?;

        let uri = Parent::<types::Atom, _>::new(&umls_config, aui).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(uri.path(), "/rest/content/current/AUI/A8345234/parents");
        assert_eq!(
            uri.query(),
            Some("pageSize=25&pageNumber=1&apiKey=some_api_key")
        );

        let batch_parents = [
            (
                atom::Batch::new(300.try_into()?, 1.try_into()?, {
                    let display_name: DisplayName = "A8345234".parse()?;
                    aui::Aui::new(display_name)
                }?),
                "pageSize=300&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(25.try_into()?, 30.try_into()?, {
                    let display_name: DisplayName = "A8345234".parse()?;
                    aui::Aui::new(display_name)
                }?),
                "pageSize=25&pageNumber=30&apiKey=some_api_key",
            ),
        ];

        for (config, query_params) in batch_parents {
            let uri = Parent::<types::Atom, _>::new(&umls_config, &config).url();
            assert_eq!(uri.scheme(), "https");
            assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
            assert_eq!(uri.path(), "/rest/content/current/AUI/A8345234/parents");
            assert_eq!(uri.query(), Some(query_params));
        }

        Ok(())
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn construct_source_concept_parent_url() -> Result<(), Box<dyn std::error::Error>> {
        let umls_config = config::Config::builder("some_api_key").build()?;
        let source_concept = SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?);

        let uri = Parent::<types::Source, _>::new(&umls_config, source_concept).url();
        assert_eq!(uri.scheme(), "https");
        assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
        assert_eq!(
            uri.path(),
            "/rest/content/current/source/SNOMEDCT_US/9468002/parents"
        );
        assert_eq!(
            uri.query(),
            Some("pageSize=25&pageNumber=1&apiKey=some_api_key")
        );

        let sets = [
            (
                atom::Batch::new(
                    300.try_into()?,
                    1.try_into()?,
                    SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?),
                ),
                "pageSize=300&pageNumber=1&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    25.try_into()?,
                    30.try_into()?,
                    SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?),
                ),
                "pageSize=25&pageNumber=30&apiKey=some_api_key",
            ),
            (
                atom::Batch::new(
                    300.try_into()?,
                    30.try_into()?,
                    SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?),
                ),
                "pageSize=300&pageNumber=30&apiKey=some_api_key",
            ),
        ];

        for (batch, query) in sets {
            let uri = Parent::<types::Source, _>::new(&umls_config, &batch).url();
            assert_eq!(uri.scheme(), "https");
            assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
            assert_eq!(
                uri.path(),
                "/rest/content/current/source/SNOMEDCT_US/9468002/parents"
            );
            assert_eq!(uri.query(), Some(query));
        }

        Ok(())
    }

    #[test]
    fn construct_search_url() -> Result<(), Box<dyn std::error::Error>> {
        let sets = [
            (
                search::Request::builder("Cough".try_into()?).build(),
                "string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .obsolete(search::core::Inclusion::Include)
                    .build(),
                "string=Cough&inputType=atom&includeObsolete=true&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .suppressible(search::core::Inclusion::Include)
                    .build(),
                "string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=true&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .input_type(InputType::Code)
                    .build(),
                "string=Cough&inputType=code&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .return_id_type(ReturnIdType::Aui)
                    .build(),
                "string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=aui&sabs=&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .try_add_source_vocabulary("SNOMEDCT_US")?
                    .build(),
                "string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=SNOMEDCT_US&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .try_add_source_vocabulary("SNOMEDCT_US")?
                    .try_add_source_vocabulary("MSH")?
                    .build(),
                "string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=SNOMEDCT_US%2CMSH&searchType=words&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .search_type(SearchType::Exact)
                    .build(),
                "string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=exact&semanticTypes=&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
            (
                search::Request::builder("Cough".try_into()?)
                    .try_add_semantic_type("Findings")?
                    .build(),
                "string=Cough&inputType=atom&includeObsolete=false&includeSuppressible=false&returnIdType=concept&sabs=&searchType=words&semanticTypes=Findings&semanticGroups=&partialSearch=false&pageSize=25&pageNumber=1&apiKey=some_api_key",
            ),
        ];

        for (request, query) in sets {
            let umls_config = config::Config::builder("some_api_key").build()?;

            let uri = Search::new(&umls_config, &request).url();
            assert_eq!(uri.scheme(), "https");
            assert_eq!(uri.host_str(), Some("uts-ws.nlm.nih.gov"));
            assert_eq!(uri.path(), "/rest/search/current");
            assert_eq!(uri.query(), Some(query));
        }

        Ok(())
    }
}
