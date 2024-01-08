//! Tracks the balance of each user and total for the tick

use crate::protocol::brc20::types::Amount;
use crate::protocol::brc20::Error;
use serde::{Deserialize, Serialize};

/// Struct that tracks the balance of token holders
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Balance {
  transferable_balance: Amount,
  total_balance: Amount,
  max: Option<Amount>,
}

impl Balance {
  pub fn new(max: Option<Amount>) -> Self {
    Self {
      transferable_balance: 0,
      total_balance: 0,
      max,
    }
  }

  pub fn decr_total(&mut self, amt: Amount) -> Result<(), Error> {
    let new_balance = if let Some(balance) = self.total_balance.checked_sub(amt) {
      balance
    } else {
      return Err(Error::BalanceUnderflow);
    };

    self.total_balance = new_balance;

    Ok(())
  }

  pub fn decr_transferable(&mut self, amt: Amount) -> Result<(), Error> {
    self.ensure_can_transfer(amt)?;

    let new_balance = if let Some(balance) = self.transferable_balance.checked_sub(amt) {
      balance
    } else {
      return Err(Error::BalanceUnderflow);
    };

    self.transferable_balance = new_balance;
    self.ensure_transfer_valid()?;

    Ok(())
  }

  pub fn incr_transferable(&mut self, amt: Amount) -> Result<(), Error> {
    let new_balance = if let Some(balance) = self.transferable_balance.checked_add(amt) {
      balance
    } else {
      return Err(Error::BalanceUnderflow);
    };

    self.ensure_below_max(new_balance)?;
    self.transferable_balance = new_balance;

    self.ensure_transfer_valid()?;

    Ok(())
  }

  pub fn incr_total(&mut self, amt: Amount) -> Result<(), Error> {
    let new_balance = if let Some(balance) = self.total_balance.checked_add(amt) {
      balance
    } else {
      return Err(Error::BalanceOverflow);
    };

    self.ensure_below_max(new_balance)?;
    self.total_balance = new_balance;

    Ok(())
  }

  fn ensure_below_max(&self, balance: Amount) -> Result<(), Error> {
    if let Some(max) = &self.max {
      if *max < balance {
        return Err(Error::ExceedsMaxBalance);
      }
    }
    Ok(())
  }

  fn ensure_can_transfer(&self, amt: Amount) -> Result<(), Error> {
    if self.total_balance < amt {
      Err(Error::TransferExceedingTotalBalance)
    } else {
      Ok(())
    }
  }

  fn ensure_transfer_valid(&self) -> Result<(), Error> {
    if self.transferable_balance > self.total_balance {
      Err(Error::InvalidAvailableBalance)
    } else {
      Ok(())
    }
  }
}
