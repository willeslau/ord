//! The basic types for BRC-20

use crate::protocol::brc20::error::messages;
use crate::protocol::InscriptionNumber;
use crate::InscriptionId;
use bitcoin::Address;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub type Tick = String;
pub type Amount = u128;
pub const MAX_BRC20_TICK_SIZE: usize = 4;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TokenInfo {
  pub token_id: TokenId,
  pub inscription_id: InscriptionId,
  pub inscription_number: InscriptionNumber,
  pub supply: u128,
  // pub minted: u128,
  // pub limit_per_mint: u128,
  // pub decimal: u8,
  // pub deployed_number: u64,
  // pub deployed_timestamp: u32,
  // pub latest_mint_number: u64,
}

#[derive(Serialize, Debug, Eq, PartialEq, Clone, Hash)]
pub struct TokenId {
  #[serde(rename = "p")]
  protocol: Protocol,
  tick: Tick,
}

/// The protocol type
#[derive(Serialize, Debug, Eq, PartialEq, Clone, Hash)]
#[repr(u8)]
pub enum Protocol {
  BRC20,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Deploy {
  #[serde(flatten)]
  pub token_id: TokenId,
  #[serde(rename = "lim", deserialize_with = "parse_u128")]
  pub limit: Amount,
  #[serde(deserialize_with = "parse_u128")]
  pub max: Amount,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Mint {
  #[serde(flatten)]
  pub token_id: TokenId,
  #[serde(rename = "amt", deserialize_with = "parse_u128")]
  pub amount: Amount,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Transfer {
  #[serde(flatten)]
  pub token_id: TokenId,
  #[serde(rename = "amt", serialize_with = "u128_serialize", deserialize_with = "parse_u128")]
  pub amount: Amount,
}

/// The BitRC-20 protocol inscription payload
#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum InscriptionPayload {
  Deploy(Deploy),
  Mint(Mint),
  Transfer(Transfer),
}

fn u128_serialize<S>(x: &u128, s: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
{
  s.serialize_str(&x.to_string())
}

fn parse_u128<'de, D>(deserializer: D) -> Result<Amount, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  Amount::from_str(&s).map_err(|_| D::Error::custom(messages::INVALID_BALANCE))
}

impl<'de> Deserialize<'de> for Protocol {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?.to_lowercase();
    Ok(match s.as_str() {
      "brc20" | "brc-20" | "0" => Protocol::BRC20,
      _ => return Err(D::Error::custom(messages::UNKNOWN_PROTOCOL)),
    })
  }
}

impl Display for Protocol {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Protocol::BRC20 => "brc-20",
      }
    )
  }
}

impl Display for TokenId {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Token(p = {}, tick={})", self.protocol, self.tick)
  }
}

impl<'de> Deserialize<'de> for TokenId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct T {
      p: Protocol,
      tick: String,
    }

    let token_id: T = T::deserialize(deserializer)?;

    if token_id.p == Protocol::BRC20 && token_id.tick.len() > MAX_BRC20_TICK_SIZE {
      return Err(D::Error::custom(messages::INVALID_TICK_LENGTH));
    }
    Ok(TokenId {
      protocol: token_id.p,
      tick: token_id.tick,
    })
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SerializableAddress {
  addr: Address,
}

impl From<Address> for SerializableAddress {
  fn from(value: Address) -> Self {
    Self { addr: value }
  }
}

impl Serialize for SerializableAddress {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let s = self.addr.to_string();
    s.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for SerializableAddress {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    let addr = Address::from_str(&s)
      .map_err(|_| D::Error::custom("cannot parse address"))?
      .assume_checked();
    Ok(addr.into())
  }
}

#[cfg(test)]
mod tests {
  use crate::protocol::brc20::types::{Deploy, InscriptionPayload, Mint, Protocol, TokenId};

  const DEPLOY: &str = r#"
    {
      "p": "brc-20",
      "op": "deploy",
      "tick": "BITUSD",
      "lim": "6250",
      "max": "100000"
    }
  "#;

  const MINT: &str = r#"
    {
      "p": "brc-20",
      "op": "mint",
      "tick": "BITUSD",
      "amt": "6250"
    }
  "#;

  #[test]
  fn test_serialization() {
    let deploy: InscriptionPayload = serde_json::from_str(DEPLOY).unwrap();
    assert_eq!(
      deploy,
      InscriptionPayload::Deploy(Deploy {
        token_id: TokenId {
          protocol: Protocol::BitRC20,
          tick: "BITUSD".to_string(),
        },
        limit: 6250,
        max: 100000,
      })
    );

    let mint: InscriptionPayload = serde_json::from_str(MINT).unwrap();
    assert_eq!(
      mint,
      InscriptionPayload::Mint(Mint {
        token_id: TokenId {
          protocol: Protocol::BitRC20,
          tick: "BITUSD".to_string(),
        },
        amount: 6250,
      })
    );
  }
}
