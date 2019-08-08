use crate::error::{Error, Result};
use log::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread::spawn,
};
use ws::{listen, Message as WsMessage, Sender};

enum Event {
    Data(u64, Vec<u8>),
    Conn(u64, Sender),
}

pub struct ServerQueue<T> {
    data_rx: Receiver<Event>,
    outs: HashMap<u64, Sender>,
    _ph: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> ServerQueue<T> {
    pub fn new(addr: &str) -> Result<Self> {
        let addr = addr.to_owned();
        let (data_tx, data_rx) = channel();

        spawn(move || {
            let mut conn_id = 0u64;

            listen(addr, |out| {
                info!("New websocket connection");

                conn_id += 1;

                let new_conn_id = conn_id;

                data_tx.send(Event::Conn(new_conn_id, out)).unwrap();

                let data_tx = data_tx.clone();

                move |msg: WsMessage| {
                    debug!("Received: {:?}", msg);

                    data_tx
                        .send(Event::Data(new_conn_id, msg.into_data()))
                        .map_err(|e| Box::new(e).into())
                }
            })
            .unwrap()
        });

        Ok(Self {
            data_rx,
            outs: HashMap::new(),
            _ph: PhantomData,
        })
    }

    fn push_raw(&mut self, conn_id: u64, bytes: Vec<u8>) -> Result<()> {
        match self.outs.get_mut(&conn_id) {
            Some(out) => {
                debug!("Sending: {:?}", out);
                out.send(bytes)?
            }
            None => {
                warn!("No connection: {}", conn_id);
                return Err(Error::Disconnected);
            }
        }

        Ok(())
    }

    fn push_raw_all(&mut self, bytes: Vec<u8>) -> Result<Vec<u64>> {
        let ids = self
            .outs
            .iter()
            .filter_map(|(id, out)| match out.send(bytes.clone()) {
                Ok(_) => None,
                Err(_) => Some(*id),
            })
            .collect();

        Ok(ids)
    }

    fn pop_raw(&mut self) -> Result<(u64, Vec<u8>)> {
        loop {
            match self.data_rx.recv()? {
                Event::Conn(conn_id, out) => {
                    debug!("New connection: {}", conn_id);
                    self.outs.insert(conn_id, out);
                }
                Event::Data(conn_id, data) => break Ok((conn_id, data)),
            }
        }
    }

    fn try_pop_raw(&mut self) -> Result<Option<(u64, Vec<u8>)>> {
        loop {
            match self.data_rx.try_recv() {
                Ok(Event::Conn(conn_id, out)) => {
                    debug!("New connection: {}", conn_id);
                    self.outs.insert(conn_id, out);
                }
                Ok(Event::Data(conn_id, data)) => break Ok(Some((conn_id, data))),
                Err(TryRecvError::Empty) => break Ok(None),
                Err(e) => break Err(e.into()),
            }
        }
    }

    pub fn push(&mut self, item: (u64, T)) -> Result<()> {
        self.push_raw(item.0, to_vec(&item.1)?)
    }

    pub fn push_all(&mut self, item: T) -> Result<Vec<u64>> {
        self.push_raw_all(to_vec(&item)?)
    }

    pub fn pop(&mut self) -> Result<(u64, T)> {
        self.pop_raw()
            .and_then(|(id, bytes)| Ok((id, from_slice(&bytes)?)))
    }

    pub fn try_pop(&mut self) -> Result<Option<(u64, T)>> {
        match self.try_pop_raw()? {
            Some((id, bytes)) => Ok(Some((id, from_slice(&bytes)?))),
            None => Ok(None),
        }
    }
}
