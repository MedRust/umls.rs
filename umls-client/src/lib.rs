#![warn(missing_docs)]

//! An asynchronous client for the UMLS REST API.
//!
//! The crate provides validated request types, high-level search pagination,
//! concept retrieval, and concept atom retrieval. Start by creating a
//! [`config::Config`], then build a request with [`search::Request`] for search
//! or [`atom::Config::builder`] for atoms.
//!
//! Enable the `reqwest` feature for request execution. Configuration contains
//! only API key, base URL, and version; attach an owned `reqwest::Client` to
//! each request with `with_client()`. Clone a client to reuse its connection pool.
//! Search and atom pagination retain the attached client.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "reqwest")]
//! use umls_http::reqwest;
//! use umls_client::{config::Config, search::Request};
//!
//! # #[cfg(feature = "reqwest")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::builder("your-api-key").build()?;
//! let response = Request::builder("cough".parse()?)
//!     .concept(&config)
//!     .with_client(reqwest::Client::default())
//!     .get()
//!     .await?;
//! let concepts = response.into_content()?;
//! println!("received {} results", concepts.list().len());
//! # Ok(())
//! # }
//! ```

/// Concept identifiers, concept resources, and concept retrieval.
pub mod concept;
/// Client configuration and configuration validation.
pub mod config;
/// Raw and high-level UMLS search APIs.
pub mod search;

pub mod atom;

/// The core compoenents shared between modules.
pub mod core;

/// Fetch concept relations.
pub mod relation;

/// Fetch conept definitions.
pub mod definition;
/// Fetch parents from UMLS.
pub mod parent;

/// Represents zero-cost abstraction types.
pub mod types {

    /// A concept url type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Concept;

    /// A atom url type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Atom;

    /// A source url type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Source;

    /// A atoms types.
    pub mod atom {
        /// Marker selecting retrieval of all atoms associated with a concept.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct All;

        /// Marker selecting retrieval of preferred atoms associated with a concept.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Preferred;
    }
}
