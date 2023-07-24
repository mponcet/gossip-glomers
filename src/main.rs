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
    fn run(&mut self) -> Result<(), NodeError>;
}

struct EchoNode {
    id: usize,
}

impl Node for EchoNode {
    fn run(&mut self) -> std::result::Result<(), NodeError> {
        let mut stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();

        // init
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

        // echo
        for line in stdin.lines() {
            let line = line.unwrap();
            let echo_msg: Message<Echo> = serde_json::from_str(&line)?;

            self.id += 1;
            let echo_ok_msg = Message {
                src: echo_msg.dst,
                dst: echo_msg.src,
                body: Body::<_> {
                    id: Some(self.id),
                    in_reply_to: echo_msg.body.id,
                    payload: EchoOk {
                        echo: echo_msg.body.payload.echo,
                    },
                },
            };
            serde_json::to_writer(&mut stdout, &echo_ok_msg)?;
            stdout.write_all(b"\n")?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut echo_node = EchoNode { id: 0 };
    echo_node.run()?;

    Ok(())
}
