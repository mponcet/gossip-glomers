use std::{
    cell::RefCell,
    io::{BufRead, Write},
};

use serde::{Deserialize, Serialize};

use crate::protocol::*;

pub enum NodeError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for NodeError {
    fn from(e: std::io::Error) -> Self {
        NodeError::Io(e)
    }
}

impl From<serde_json::Error> for NodeError {
    fn from(e: serde_json::Error) -> Self {
        NodeError::Json(e)
    }
}

impl std::error::Error for NodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NodeError::Io(e) => Some(e),
            NodeError::Json(e) => Some(e),
        }
    }
}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeError::Io(_) => write!(f, "IO error"),
            NodeError::Json(_) => write!(f, "Serialization/Deserialization error"),
        }
    }
}
impl std::fmt::Debug for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{:?}\n", e.source())?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

pub trait Node {
    type PayloadIn;
    type PayloadOut;

    fn reply(
        &mut self,
        runtime: &Runtime<Self::PayloadIn, Self::PayloadOut>,
        msg: Self::PayloadIn,
    ) -> Self::PayloadOut;
}

pub struct Runtime<I, O> {
    id: usize,
    pub node_id: Option<String>,
    handler: RefCell<Box<dyn Node<PayloadIn = I, PayloadOut = O>>>,
}

pub struct RuntimeBuilder<H, S> {
    handler: H,
    state: std::marker::PhantomData<S>,
}

// States
pub struct Handler<I, O>(Box<dyn Node<PayloadIn = I, PayloadOut = O>>);
pub struct NoHandler;

pub struct NotRunnable;
pub struct Runnable;

impl RuntimeBuilder<NoHandler, NotRunnable> {
    pub fn new() -> Self {
        RuntimeBuilder {
            handler: NoHandler,
            state: std::marker::PhantomData,
        }
    }

    pub fn with_handler<I, O>(
        self,
        handler: Box<dyn Node<PayloadIn = I, PayloadOut = O>>,
    ) -> RuntimeBuilder<Handler<I, O>, Runnable> {
        RuntimeBuilder {
            handler: Handler(handler),
            state: std::marker::PhantomData,
        }
    }
}

impl Default for RuntimeBuilder<NoHandler, NotRunnable> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, O> RuntimeBuilder<Handler<I, O>, Runnable> {
    pub fn build(self) -> Runtime<I, O> {
        Runtime {
            id: 0,
            node_id: None,
            handler: RefCell::new(self.handler.0),
        }
    }
}

impl<I, O> Runtime<I, O>
where
    I: for<'a> Deserialize<'a>,
    O: Serialize,
{
    pub fn run(mut self) -> Result<(), NodeError> {
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let init_msg: Message<Init> = serde_json::from_str(&line)?;

        self.node_id = Some(init_msg.body.payload.node_id);
        let init_ok_msg = Message {
            src: init_msg.dst,
            dst: init_msg.src,
            body: Body::<_> {
                id: Some(self.id),
                in_reply_to: init_msg.body.id,
                payload: InitOk {},
            },
        };
        serde_json::to_writer(&mut stdout, &init_ok_msg)?;
        stdout.write_all(b"\n")?;

        for line in stdin.lines() {
            let line = line.unwrap();
            let msg_in: Message<I> = serde_json::from_str(&line)?;

            self.id += 1;
            let echo_ok_msg = Message {
                src: msg_in.dst,
                dst: msg_in.src,
                body: Body::<_> {
                    id: Some(self.id),
                    in_reply_to: msg_in.body.id,
                    payload: self.handler.borrow_mut().reply(&self, msg_in.body.payload),
                },
            };
            serde_json::to_writer(&mut stdout, &echo_ok_msg)?;
            stdout.write_all(b"\n")?;
        }

        Ok(())
    }
}
