use gossip_glomers::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "generate")]
struct Generate {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename = "generate_ok")]
struct GenerateOk {
    id: Uuid,
}

struct UniqueIdsNode {}

impl Node for UniqueIdsNode {
    type PayloadIn = Generate;
    type PayloadOut = GenerateOk;

    fn reply(&mut self, _msg: Generate) -> GenerateOk {
        GenerateOk { id: Uuid::new_v4() }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let unique_ids_node = UniqueIdsNode {};
    let runtime = RuntimeBuilder::new()
        .with_handler(Box::new(unique_ids_node))
        .build();
    let _ = runtime.run();

    Ok(())
}
