//! The BRC20 inscription event handler

mod balance;
mod error;
mod storage;
mod tracker;
mod types;

pub use crate::protocol::brc20::error::Error;
pub use crate::protocol::brc20::tracker::Tracker;
use crate::protocol::brc20::types::InscriptionPayload;
use crate::protocol::Result;
use crate::protocol::{InscriptionEventHandler, NewInscription, TransferInscription};
use std::cell::RefCell;
pub use storage::*;

pub struct BRC20InscriptionHandler<'a, 'db, 'tx> {
  pub(crate) tracker: RefCell<Tracker<'a, 'db, 'tx>>,
}

impl<'a, 'db, 'tx> BRC20InscriptionHandler<'a, 'db, 'tx> {
  pub fn new(tracker: Tracker<'a, 'db, 'tx>) -> Self {
    Self {
      tracker: RefCell::new(tracker),
    }
  }
}

impl<'a, 'db, 'tx> InscriptionEventHandler for BRC20InscriptionHandler<'a, 'db, 'tx> {
  fn handle_new(&self, event: &NewInscription) -> Result<()> {
    if let Some(body) = &event.inscription.body {
      let payload = serde_json::from_slice::<InscriptionPayload>(body).map_err(Error::from)?;
      log::debug!("payload received: {payload:?}");

      let mut tracker = self.tracker.borrow_mut();
      match payload {
        InscriptionPayload::Deploy(p) => tracker.deploy(&event.owner, p)?,
        InscriptionPayload::Mint(p) => tracker.mint(&event.owner, p)?,
        InscriptionPayload::Transfer(p) => {
          tracker.inscribe_transfer(&event.owner, event.inscription_id, p)?
        }
      }
    } else {
      log::debug!(
        "inscription has no  brc-20 body in {}",
        event.inscription_id
      );
    }
    Ok(())
  }

  fn handle_transfer(&self, event: &TransferInscription) -> Result<()> {
    let mut tracker = self.tracker.borrow_mut();
    tracker.transfer(&event.from, &event.to, event.inscription_id)?;
    Ok(())
  }
}
