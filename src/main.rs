use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Message<Payload> {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body<Payload>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct Body<Payload> {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "init")]
struct Init {
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "init_ok")]
struct InitOk {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "echo")]
struct Echo {
    echo: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "echo_ok")]
struct EchoOk {
    echo: String,
}

enum NodeError {
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

trait Node {
    type PayloadIn;
    type PayloadOut;

    fn reply(&mut self, msg: Self::PayloadIn) -> Self::PayloadOut;
}

struct Runtime<I, O> {
    id: usize,
    handler: Box<dyn Node<PayloadIn = I, PayloadOut = O>>,
}

struct RuntimeBuilder<H, S> {
    handler: H,
    state: std::marker::PhantomData<S>,
}

// States
struct Handler<I, O>(Box<dyn Node<PayloadIn = I, PayloadOut = O>>);
struct NoHandler;

struct NotRunnable;
struct Runnable;

impl RuntimeBuilder<NoHandler, NotRunnable> {
    fn new() -> Self {
        RuntimeBuilder {
            handler: NoHandler,
            state: std::marker::PhantomData,
        }
    }

    fn with_handler<I, O>(
        self,
        handler: Box<dyn Node<PayloadIn = I, PayloadOut = O>>,
    ) -> RuntimeBuilder<Handler<I, O>, Runnable> {
        RuntimeBuilder {
            handler: Handler(handler),
            state: std::marker::PhantomData,
        }
    }
}

impl<I, O> RuntimeBuilder<Handler<I, O>, Runnable> {
    fn build(self) -> Runtime<I, O> {
        Runtime {
            id: 0,
            handler: self.handler.0,
        }
    }
}

impl<I, O> Runtime<I, O>
where
    I: for<'a> Deserialize<'a>,
    O: Serialize,
{
    fn run(mut self) -> Result<(), NodeError> {
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let init_msg: Message<Init> = serde_json::from_str(&line)?;

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
            let echo_msg: Message<I> = serde_json::from_str(&line)?;

            self.id += 1;
            let echo_ok_msg = Message {
                src: echo_msg.dst,
                dst: echo_msg.src,
                body: Body::<_> {
                    id: Some(self.id),
                    in_reply_to: echo_msg.body.id,
                    payload: self.handler.reply(echo_msg.body.payload),
                },
            };
            serde_json::to_writer(&mut stdout, &echo_ok_msg)?;
            stdout.write_all(b"\n")?;
        }

        Ok(())
    }
}

struct EchoNode;

impl Node for EchoNode {
    type PayloadIn = Echo;
    type PayloadOut = EchoOk;

    fn reply(&mut self, msg: Echo) -> EchoOk {
        EchoOk { echo: msg.echo }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let echo_node = EchoNode {};
    let runtime = RuntimeBuilder::new()
        .with_handler(Box::new(echo_node))
        .build();
    let _ = runtime.run();

    Ok(())
}
