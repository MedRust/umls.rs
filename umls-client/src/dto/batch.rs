use std::num::NonZeroU64;

/// A value paired with non-zero pagination parameters.
#[derive(Debug, Clone)]
pub struct Batch<T> {
    size: NonZeroU64,
    number: NonZeroU64,
    value: T,
}

impl<T> Batch<T> {
    /// Creates a paginated value with the supplied page size and page number.
    pub fn new(size: NonZeroU64, number: NonZeroU64, value: T) -> Self {
        Self {
            size,
            number,
            value,
        }
    }

    /// Returns the maximum number of atoms requested per page.
    pub fn size(&self) -> std::num::NonZero<u64> {
        self.size
    }

    /// Returns the one-based page number.
    pub fn number(&self) -> std::num::NonZero<u64> {
        self.number
    }

    /// Returns the value associated with these pagination parameters.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Increments the page number by one.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Overflow`] if incrementing would exceed
    /// [`u64::MAX`].
    pub fn checked_next(&mut self) -> Result<(), BatchError> {
        let number = self
            .number
            .get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(BatchError::Overflow)?;

        self.number = number;
        Ok(())
    }

    /// Decrements the page number by one.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::NoPreviousPage`] when already on the first page.
    pub fn checked_previous(&mut self) -> Result<(), BatchError> {
        let number = self
            .number
            .get()
            .checked_sub(1)
            .and_then(NonZeroU64::new)
            .ok_or(BatchError::NoPreviousPage)?;

        self.number = number;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
/// An error produced while changing the requested atom page.
pub enum BatchError {
    /// No next page exists, or incrementing the page number would exceed
    /// [`u64::MAX`].
    #[error("Page overflow")]
    Overflow,

    /// The first page has no preceding page.
    #[error("No previous page")]
    NoPreviousPage,
}
