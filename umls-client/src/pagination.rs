//! Pagination utilities for high‑level client usage.
//!
//! The implementation lives in `search::page::Page`, but exposing it
//! through this module prevents name collisions with the DTO `Page` type.

/// Pagination wrapper for search results.
pub type Page<T, C> = crate::search::Page<T, C>;
