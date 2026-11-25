use std::num::NonZeroU64;

/// The default non zero page size.
pub const DEFAULT_PAGE_SIZE: NonZeroU64 = NonZeroU64::new(25).unwrap();

/// The default non zero page number.
pub const DEFAULT_PAGE_NUMBER: NonZeroU64 = NonZeroU64::new(1).unwrap();
