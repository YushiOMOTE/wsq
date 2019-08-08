use crate::error::Result;
use log::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use std::{
    marker::PhantomData,
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread::spawn,
};
use ws::{connect, Message as WsMessage, Sender};

pub struct ClientQueue<T> {
    rx: Receiver<Vec<u8>>,
    out: Sender,
    _ph: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> ClientQueue<T> {
    pub fn new(addr: &str) -> Result<Self> {
        let addr = addr.to_owned();
        let (data_tx, data_rx) = channel();
        let (conn_tx, conn_rx) = channel();

        spawn(move || {
            connect(addr, |out| {
                debug!("Connected");

                conn_tx.send(out).unwrap();

                let data_tx = data_tx.clone();

                move |msg: WsMessage| {
                    debug!("Received: {:?}", msg);

                    data_tx
                        .send(msg.into_data())
                        .map_err(|e| Box::new(e).into())
                }
            })
            .unwrap()
        });

        Ok(Self {
            rx: data_rx,
            out: conn_rx.recv().unwrap(),
            _ph: PhantomData,
        })
    }

    fn push_raw(&self, bytes: Vec<u8>) -> Result<()> {
        debug!("Sending: {:?}", bytes);

        self.out.send(bytes)?;

        Ok(())
    }

    fn try_pop_raw(&self) -> Result<Option<Vec<u8>>> {
        Ok(self
            .rx
            .try_recv()
            .map(|msg| Some(msg))
            .or_else(|e| match e {
                TryRecvError::Empty => Ok(None),
                e => Err(e),
            })?)
    }

    fn pop_raw(&self) -> Result<Vec<u8>> {
        Ok(self.rx.recv()?)
    }

    pub fn push(&self, item: T) -> Result<()> {
        let bytes = to_vec(&item)?;

        self.push_raw(bytes)
    }

    pub fn pop(&self) -> Result<T> {
        self.pop_raw().and_then(|bytes| Ok(from_slice(&bytes)?))
    }

    pub fn try_pop(&self) -> Result<Option<T>> {
        let bytes = match self.try_pop_raw()? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        Ok(Some(from_slice(&bytes)?))
    }
}
