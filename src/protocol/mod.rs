//! The generic inscription event handler

pub mod brc20;
pub mod error;
pub mod handler;
pub mod storage;

use crate::protocol::error::Error;
use crate::{Inscription, InscriptionId, SatPoint};
use bitcoin::{Address, OutPoint};

pub type InscriptionNumber = i32;
pub type Result<T> = std::result::Result<T, Error>;

/// A new inscription is made
#[derive(Debug, Clone)]
pub enum InscriptionEvent {
  New {
    prev_txn_outpoint: OutPoint,
    inscription_id: InscriptionId,
    satpoint: SatPoint,
  },
  Transfer {
    prev_satpoint: SatPoint,
    new_satpoint: SatPoint,
    inscription_id: InscriptionId,
  },
}

/// A new inscription is made
#[derive(Debug, Clone)]
pub struct NewInscription {
  pub prev_txn_outpoint: OutPoint,
  pub satpoint: SatPoint,
  pub owner: Address,

  pub inscription_id: InscriptionId,
  pub inscription: Inscription,
}

/// An existing inscription is transferred
#[derive(Debug, Clone)]
pub struct TransferInscription {
  pub prev_satpoint: SatPoint,
  pub new_satpoint: SatPoint,

  /// The inscription that is transferred from
  pub from: Address,
  /// The inscription that is transferred to
  pub to: Address,

  pub inscription_id: InscriptionId,
}

pub trait InscriptionEventHandler {
  /// Called when a new inscription is made
  fn handle_new(&self, event: &NewInscription) -> Result<()>;

  /// Called when an existing inscription is transferred
  fn handle_transfer(&self, event: &TransferInscription) -> Result<()>;
}

impl InscriptionEvent {
  pub fn inscription_id(&self) -> &InscriptionId {
    match self {
      InscriptionEvent::New { inscription_id, .. } => inscription_id,
      InscriptionEvent::Transfer { inscription_id, .. } => inscription_id,
    }
  }

  pub fn prev_outpoint(&self) -> &OutPoint {
    match self {
      InscriptionEvent::New {
        prev_txn_outpoint: prev_txn_out,
        ..
      } => prev_txn_out,
      InscriptionEvent::Transfer { prev_satpoint, .. } => &prev_satpoint.outpoint,
    }
  }

  pub fn current_outpoint(&self) -> &OutPoint {
    match self {
      InscriptionEvent::New { satpoint, .. } => &satpoint.outpoint,
      InscriptionEvent::Transfer { new_satpoint, .. } => &new_satpoint.outpoint,
    }
  }
}
