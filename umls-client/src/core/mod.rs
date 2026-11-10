//! Core UMLS domain identifiers and endpoint URL utilities.
//!
//! This module provides domain newtypes for unique identifiers used in the UMLS
//! ecosystem, along with URL builders and validation logic:
//! - [`cui`]: Concept Unique Identifier ([`Cui`](cui::Cui)).
//! - [`aui`]: Atom Unique Identifier ([`Aui`](aui::Aui)).
//! - [`url`]: Endpoint URL construction and safety validation.
//!
//! # Example
//!
//! ```
//! use umls_client::core::cui::Cui;
//!
//! let cui = Cui::new("C0010200")?;
//! assert_eq!(cui.as_str(), "C0010200");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Validated UMLS Concept Unique Identifiers (CUIs).
pub mod cui;

/// An Atom unique Identifiers implementation.
pub mod aui;

/// URL construction API.
pub mod url;
