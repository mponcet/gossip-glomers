use gossip_glomers::*;

use serde::{Deserialize, Serialize};

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
