//! The error for all the protocols

use crate::protocol::brc20::Error as BRC20Error;
use bitcoin::OutPoint;
use thiserror::Error;

/// Errors that does not block inscription handling event loop
#[derive(Debug, Error)]
pub enum NonBlockingError {
  #[error("Brc20 protocol error {0}")]
  BRC20(BRC20Error),
  /// This means a bug has happened and it's not critical
  #[error("Unknown error")]
  Bug,
}

#[derive(Debug, Error)]
pub enum BlockingError {
  #[error("Cannot perform storage related functions {0}")]
  Storage(redb::StorageError),
  #[error("Outpoint not found in storage: {0}")]
  OutpointNotFound(OutPoint),
  #[error("Invalid address network: {0}")]
  InvalidAddressNetwork(bitcoin::address::Error),
}

#[derive(Debug, Error)]
pub enum Error {
  #[error("Non-blocking error {0}")]
  NonBlocking(NonBlockingError),
  #[error("Blocking error encountered {0}")]
  Blocking(BlockingError),
}

impl From<BRC20Error> for Error {
  fn from(e: BRC20Error) -> Self {
    Self::NonBlocking(NonBlockingError::BRC20(e))
  }
}

impl Error {
  pub fn nonblocking_bug() -> Self {
    Self::NonBlocking(NonBlockingError::Bug)
  }
}

impl From<redb::StorageError> for Error {
  fn from(err: redb::StorageError) -> Self {
    Self::Blocking(BlockingError::Storage(err))
  }
}

impl From<bitcoin::address::Error> for Error {
  fn from(err: bitcoin::address::Error) -> Self {
    Self::Blocking(BlockingError::InvalidAddressNetwork(err))
  }
}
