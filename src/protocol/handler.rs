//! The struct that calls all the inscription handler.

use crate::inscriptions::ParsedEnvelope;
use crate::protocol::brc20::BRC20InscriptionHandler;
use crate::protocol::error::{BlockingError, Error};
use crate::protocol::storage::{OutpointToAddressTable, ProtocolStorage};
use crate::protocol::Result;
use crate::protocol::{
  InscriptionEvent, InscriptionEventHandler, NewInscription, TransferInscription,
};
use crate::Inscription;
use bitcoin::{Address, OutPoint, Transaction, TxOut, Txid};
use std::cell::RefCell;
use std::collections::HashMap;

pub enum Handler<'a, 'db, 'tx> {
  BRC20(BRC20InscriptionHandler<'a, 'db, 'tx>),
}

impl<'a, 'db, 'tx> InscriptionEventHandler for Handler<'a, 'db, 'tx> {
  fn handle_new(&self, event: &NewInscription) -> Result<()> {
    match self {
      Handler::BRC20(h) => h.handle_new(event),
    }
  }

  fn handle_transfer(&self, event: &TransferInscription) -> Result<()> {
    match self {
      Handler::BRC20(h) => h.handle_transfer(event),
    }
  }
}

/// Handles the inscription events
pub struct InscriptionManager<'a, 'db, 'tx> {
  /// The list of handlers registered
  handlers: Vec<Handler<'a, 'db, 'tx>>,
  /// The parsed inscription events for the transactions
  events: HashMap<Txid, Vec<InscriptionEvent>>,
  /// The list of transactions for the block
  txns: Vec<Transaction>,
  /// Handles the storage related functions
  storage: RefCell<ProtocolStorage<'a, 'db, 'tx>>,
  /// The network it is targeting
  network: bitcoin::Network,
}

impl<'a, 'db, 'tx> InscriptionManager<'a, 'db, 'tx> {
  pub fn new_in_regtest(
    outpoint_to_script: &'a mut OutpointToAddressTable<'db, 'tx>,
    handlers: Vec<Handler<'a, 'db, 'tx>>,
    txns: Vec<Transaction>,
  ) -> Self {
    let storage = RefCell::new(ProtocolStorage::new(outpoint_to_script));
    Self {
      storage,
      handlers,
      txns,
      events: HashMap::new(),
      network: bitcoin::Network::Regtest,
    }
  }

  #[allow(dead_code)]
  pub fn new_with_network(
    network: bitcoin::Network,
    outpoint_to_script: &'a mut OutpointToAddressTable<'db, 'tx>,
    handlers: Vec<Handler<'a, 'db, 'tx>>,
    txns: Vec<Transaction>,
  ) -> Self {
    let storage = RefCell::new(ProtocolStorage::new(outpoint_to_script));
    Self {
      storage,
      handlers,
      txns,
      events: HashMap::new(),
      network,
    }
  }

  pub fn record_event(&mut self, txid: Txid, event: InscriptionEvent) {
    if let Some(events) = self.events.get_mut(&txid) {
      events.push(event);
    } else {
      self.events.insert(txid, vec![event]);
    }
  }

  pub fn process(&self) -> Result<()> {
    for txn in &self.txns {
      // skip coinbase transaction.
      if txn
        .input
        .first()
        .map(|tx_in| tx_in.previous_output.is_null())
        .unwrap_or_default()
      {
        continue;
      }
      self.process_txn(txn)?;
    }
    Ok(())
  }

  fn process_txn(&self, txn: &Transaction) -> Result<()> {
    log::debug!("processing txn: {}", txn.txid());

    let inscriptions = ParsedEnvelope::from_transaction(txn)
      .into_iter()
      .map(|e| e.payload)
      .collect::<Vec<_>>();

    let txid = txn.txid();

    // process inscription events.
    if let Some(events) = self.events.get(&txid) {
      // track the inscription ownerships.
      // TODO: add filtering of events, to see if an event should be tracked in storage
      let ownerships = self.track_outpoint_ownerships(events, &txn.output)?;

      for input in &txn.input {
        for event in events {
          if *event.prev_outpoint() != input.previous_output {
            break;
          }

          self.process_event(event.clone(), &inscriptions, &ownerships)?;
        }
      }
    }
    Ok(())
  }

  fn process_event(
    &self,
    event: InscriptionEvent,
    inscriptions: &[Inscription],
    ownerships: &HashMap<OutPoint, Address>,
  ) -> Result<()> {
    log::debug!("processing event: {event:?}");

    match event {
      InscriptionEvent::New {
        prev_txn_outpoint,
        inscription_id,
        satpoint,
      } => {
        let inscription = inscriptions
          .get(event.inscription_id().index as usize)
          .ok_or(Error::nonblocking_bug())?
          .clone();
        let owner = ownerships[&satpoint.outpoint].clone();
        let event = NewInscription {
          prev_txn_outpoint,
          satpoint,
          owner,
          inscription_id,
          inscription,
        };
        self.handle_new(&event)
      }
      InscriptionEvent::Transfer {
        prev_satpoint,
        new_satpoint,
        inscription_id,
      } => {
        let to = ownerships[&new_satpoint.outpoint].clone();
        let from = self
          .storage
          .borrow()
          .get_script_from_outpoint(prev_satpoint.outpoint)?
          .ok_or(Error::Blocking(BlockingError::OutpointNotFound(
            prev_satpoint.outpoint,
          )))?;
        let event = TransferInscription {
          prev_satpoint,
          new_satpoint,
          from,
          to,
          inscription_id,
        };
        self.handle_transfer(&event)
      }
    }
  }

  fn track_outpoint_ownerships(
    &self,
    events: &[InscriptionEvent],
    outpoints: &[TxOut],
  ) -> Result<HashMap<OutPoint, Address>> {
    let mut map = HashMap::new();
    for event in events {
      let cur_output = event.current_outpoint();
      if let Some(out) = outpoints.get(cur_output.vout as usize) {
        let script = out.script_pubkey.as_script();
        let address = Address::from_script(script, self.network)?;
        map.insert(*cur_output, address);
      } else {
        log::error!("bug, event outpoint not found in txn outpoints");
        return Err(Error::Blocking(BlockingError::OutpointNotFound(
          *cur_output,
        )));
      }
    }

    if !map.is_empty() {
      let mut storage = self.storage.borrow_mut();
      for (outpoint, address) in map.iter() {
        storage.store_outpoint_to_script(*outpoint, address.clone())?;
      }
    }

    Ok(map)
  }

  fn handle_new(&self, event: &NewInscription) -> Result<()> {
    for h in self.handlers.iter() {
      match h.handle_new(event) {
        Ok(_) => {}
        Err(Error::NonBlocking(e)) => {
          log::debug!("non blocking error: {e}");
        }
        Err(Error::Blocking(e)) => {
          log::error!("blocking error encountered: {e}");
          return Err(Error::Blocking(e));
        }
      }
      continue;
    }
    Ok(())
  }

  fn handle_transfer(&self, event: &TransferInscription) -> Result<()> {
    for h in self.handlers.iter() {
      h.handle_transfer(event)?;
    }
    Ok(())
  }
}
