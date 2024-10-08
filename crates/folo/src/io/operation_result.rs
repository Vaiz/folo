use crate::io::PinnedBuffer;
use thiserror::Error;

/// An error for an I/O operation that was attempted on a data buffer. Contains not only the error
/// information but also the data buffer that was used, enabling it to be inspected or reused.
///
/// If you do not care about the buffer, simply call `into_inner()` to extract the inner error.
#[derive(Debug, Error)]
#[error("I/O operation failed: {inner}")]
pub struct OperationError {
    pub inner: crate::io::Error,
    pub buffer: PinnedBuffer,
}

impl OperationError {
    pub fn new(inner: crate::io::Error, buffer: PinnedBuffer) -> Self {
        Self { inner, buffer }
    }

    pub fn into_inner(self) -> crate::io::Error {
        self.inner
    }

    pub fn into_inner_and_buffer(self) -> (crate::io::Error, PinnedBuffer) {
        (self.inner, self.buffer)
    }
}

pub type OperationResult = std::result::Result<PinnedBuffer, OperationError>;

pub trait OperationResultExt {
    fn into_inner(self) -> crate::io::Result<PinnedBuffer>;
}

impl OperationResultExt for OperationResult {
    fn into_inner(self) -> crate::io::Result<PinnedBuffer> {
        match self {
            Ok(buffer) => Ok(buffer),
            Err(OperationError { inner, .. }) => Err(inner),
        }
    }
}
