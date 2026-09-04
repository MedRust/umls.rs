#![warn(missing_docs)]

//! An asynchronous client for the UMLS REST API.
//!
//! The crate provides validated request types, high-level search pagination,
//! and concept retrieval. Start by creating a [`config::Config`], then build a
//! request with [`search::Request`].
//!
//! # Example
//!
//! ```no_run
//! use umls_client::{config::Config, search::Request};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::builder("your-api-key").build()?;
//! let response = Request::builder("cough".parse()?)
//!     .concept(&config)
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
