//! BRC-20 related errors

use crate::protocol::brc20::types::TokenId;
use thiserror::Error;

pub mod messages {
  pub const UNKNOWN_PROTOCOL: &str = "unknown protocol";
  pub const INVALID_BALANCE: &str = "invalid balance";
  pub const INVALID_TICK_LENGTH: &str = "invalid tick length";
}

#[derive(Debug, Error)]
pub enum Error {
  #[error("Transfer amount exceeds total balance")]
  TransferExceedingTotalBalance,
  #[error("Available balance exceeded total balance")]
  InvalidAvailableBalance,
  #[error("Balance exceeds max allowed balance")]
  ExceedsMaxBalance,
  #[error("Balance is overflow")]
  BalanceOverflow,
  #[error("Balance is underflow")]
  BalanceUnderflow,
  #[error("Token does not exist {0}")]
  TokenNotExists(TokenId),
  #[error("Duplicated token deployment {0}")]
  DuplicatedTokenDeployment(TokenId),
  #[error("The tick has exceeded max length")]
  InvalidTickLength,
  #[error("The balance is not valid")]
  InvalidBalance,
  #[error("The protocol is not supported")]
  UnknownProtocol,
  #[error("The inscription payload for brc20 is invalid")]
  InvalidInscriptionPayload,
  #[error("Storage error")]
  Storage(redb::StorageError),
}

impl From<redb::StorageError> for Error {
  fn from(value: redb::StorageError) -> Self {
    Self::Storage(value)
  }
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    match e.to_string().as_str() {
      messages::UNKNOWN_PROTOCOL => Self::UnknownProtocol,
      messages::INVALID_BALANCE => Self::InvalidBalance,
      messages::INVALID_TICK_LENGTH => Self::InvalidTickLength,
      _ => Self::InvalidInscriptionPayload,
    }
  }
}
