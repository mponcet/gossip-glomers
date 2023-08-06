use gossip_glomers::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "generate")]
struct Generate {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "generate_ok")]
struct GenerateOk {
    id: String,
}

struct UniqueIdsNode {
    id: usize,
}

impl Node for UniqueIdsNode {
    type PayloadIn = Generate;
    type PayloadOut = GenerateOk;

    fn reply(
        &mut self,
        runtime: &Runtime<Self::PayloadIn, Self::PayloadOut>,
        _msg: Generate,
    ) -> GenerateOk {
        self.id += 1;
        GenerateOk {
            id: format!("{}{}", runtime.node_id.as_ref().unwrap(), self.id),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let unique_ids_node = UniqueIdsNode { id: 0 };
    let runtime = RuntimeBuilder::new()
        .with_handler(Box::new(unique_ids_node))
        .build();
    let _ = runtime.run();

    Ok(())
}
