//! Storage related functions

use crate::protocol::Result;
use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::{Address, OutPoint};
use redb::{ReadableTable, RedbKey, RedbValue, Table, TableDefinition, TypeName};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::io::Cursor;
use std::str::FromStr;

pub const OUTPOINT_TO_ADDRESS_TABLE: TableDefinition<OutpointKey, AddressValue> =
  TableDefinition::new("PROTOCOL_OUTPOINT_TO_ADDRESS");

pub type OutpointToAddressTable<'db, 'tx> = Table<'db, 'tx, OutpointKey, AddressValue>;

/// The outpoint storage key wrapper
#[derive(Debug)]
pub struct OutpointKey {
  inner: OutPoint,
}

/// The script buf storage wrapper
#[derive(Debug)]
pub struct AddressValue {
  inner: Address,
}

pub(crate) struct ProtocolStorage<'a, 'db, 'tx> {
  outpoint_to_address: &'a mut OutpointToAddressTable<'db, 'tx>,
}

impl<'a, 'db, 'tx> ProtocolStorage<'a, 'db, 'tx> {
  pub(crate) fn new(outpoint_to_address: &'a mut OutpointToAddressTable<'db, 'tx>) -> Self {
    Self {
      outpoint_to_address,
    }
  }

  pub fn store_outpoint_to_script(&mut self, outpoint: OutPoint, address: Address) -> Result<()> {
    let key = OutpointKey { inner: outpoint };
    let val = AddressValue { inner: address };
    self.outpoint_to_address.insert(key, val)?;
    Ok(())
  }

  pub fn get_script_from_outpoint(&self, outpoint: OutPoint) -> Result<Option<Address>> {
    let key = OutpointKey { inner: outpoint };
    Ok(self.outpoint_to_address.get(key)?.map(|a| a.value().inner))
  }
}

impl RedbValue for OutpointKey {
  type SelfType<'a> = OutpointKey where Self: 'a;
  type AsBytes<'a> = Vec<u8> where Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    let mut reader = Cursor::new(data);
    let output = OutPoint::consensus_decode(&mut reader).unwrap();
    Self { inner: output }
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    let mut writer = Vec::new();
    value.inner.consensus_encode(&mut writer).unwrap();
    writer
  }

  fn type_name() -> TypeName {
    TypeName::new("protocol::OutpointKey")
  }
}

impl RedbKey for OutpointKey {
  fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
    data1.cmp(data2)
  }
}

impl RedbValue for AddressValue {
  type SelfType<'a> = AddressValue where Self: 'a;
  type AsBytes<'a> = Vec<u8> where Self: 'a;

  fn fixed_width() -> Option<usize> {
    None
  }

  fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
  where
    Self: 'a,
  {
    let s = String::from_utf8(Vec::from(data)).expect("cannot convert vec u8 to string");
    let inner = Address::from_str(&s)
      .expect("cannot parse address")
      .assume_checked();
    AddressValue { inner }
  }

  fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
  where
    Self: 'a,
    Self: 'b,
  {
    value.inner.to_string().into_bytes()
  }

  fn type_name() -> TypeName {
    TypeName::new("protocol::AddressValue")
  }
}
