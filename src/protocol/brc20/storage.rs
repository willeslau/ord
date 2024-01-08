//! The storage related functions and types for brc-20

use crate::protocol::brc20::balance::Balance;
use crate::protocol::brc20::types::{TokenId, Transfer};
use crate::InscriptionId;
use redb::{RedbKey, RedbValue, Table, TableDefinition, TypeName};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBalanceKey {
  pub(crate) token: TokenId,
  pub(crate) owner: String,
}

pub const BRC20_USER_BALANCE_TABLE: TableDefinition<UserBalanceKey, Balance> =
  TableDefinition::new("BRC20_USER_BALANCE_TABLE");
pub const BRC20_TOKEN_BALANCE_TABLE: TableDefinition<TokenId, Balance> =
  TableDefinition::new("BRC20_TOKEN_BALANCE_TABLE");
pub const BRC20_TRANSFER_TABLE: TableDefinition<InscriptionIdKey, Transfer> =
  TableDefinition::new("BRC20_TRANSFER");

pub type TransferTable<'db, 'tx> = Table<'db, 'tx, InscriptionIdKey, Transfer>;
pub type UserBalanceTable<'db, 'tx> = Table<'db, 'tx, UserBalanceKey, Balance>;
pub type TokenBalanceTable<'db, 'tx> = Table<'db, 'tx, TokenId, Balance>;

// The impl below are all dummy implementation

impl RedbValue for TokenId {
  type SelfType<'a> = TokenId where Self: 'a;
  type AsBytes<'a> = Vec<u8> where Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    serde_json::from_slice(data).unwrap()
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    serde_json::to_vec(value).unwrap()
  }

  fn type_name() -> TypeName {
    TypeName::new("protocol::brc20::TokenId")
  }
}

impl RedbKey for TokenId {
  fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
    data1.cmp(data2)
  }
}

impl RedbKey for UserBalanceKey {
  fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
    data1.cmp(data2)
  }
}

impl RedbValue for UserBalanceKey {
  type SelfType<'a> = UserBalanceKey where Self: 'a;
  type AsBytes<'a> = Vec<u8> where Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    serde_json::from_slice(data).unwrap()
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    serde_json::to_vec(value).unwrap()
  }

  fn type_name() -> TypeName {
    TypeName::new("protocol::brc20::UserBalanceKey")
  }
}

impl RedbValue for Balance {
  type SelfType<'a> = Balance where Self: 'a;
  type AsBytes<'a> = Vec<u8> where Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    serde_json::from_slice(data).unwrap()
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    serde_json::to_vec(value).unwrap()
  }

  fn type_name() -> TypeName {
    TypeName::new("protocol::brc20::Balance")
  }
}

#[derive(Debug)]
pub struct InscriptionIdKey {
  pub(crate) inner: InscriptionId,
}

impl RedbKey for InscriptionIdKey {
  fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
    data1.cmp(data2)
  }
}

impl RedbValue for InscriptionIdKey {
  type SelfType<'a> = InscriptionIdKey where Self: 'a;
  type AsBytes<'a> = Vec<u8> where Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    let s = String::from_utf8(Vec::from(data))
      .expect("inscription id key cannot convert vec u8 to string");
    let inner = InscriptionId::from_str(&s).expect("invalid inscription id key from string ");
    InscriptionIdKey { inner }
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    value.inner.to_string().into_bytes()
  }

  fn type_name() -> TypeName {
    TypeName::new("protocol::brc20::InscriptionIdKey")
  }
}

impl RedbValue for Transfer {
  type SelfType<'a> = Transfer where Self: 'a;
  type AsBytes<'a> = Vec<u8> where Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    serde_json::from_slice(data).unwrap()
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    serde_json::to_vec(value).unwrap()
  }

  fn type_name() -> TypeName {
    TypeName::new("protocol::brc20::InscriptionValue")
  }
}
