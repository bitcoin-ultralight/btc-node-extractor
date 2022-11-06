mod bitcoin_rpc;
mod jsonrpc;

use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crate::bitcoin_rpc::BitcoinRpc;

fn get_parent_hash(header: &[u8; 80]) -> anyhow::Result<String> {
    let mut parent_hash = header[4..][..32].to_vec();
    parent_hash.reverse();
    Ok(hex::encode(parent_hash))
}

const ZERO_BLOCK_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc = BitcoinRpc::new_localhost()?;

    let mut block_hash = rpc.get_best_block_hash().await?;

    let mut blocks = Vec::<Box<[u8; 80]>>::new();

    let mut i = 0;

    while block_hash != ZERO_BLOCK_HASH {
        let block_header = rpc
            .get_binary_block_header(&block_hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing block {}", block_hash))?;
        block_hash = get_parent_hash(&block_header)?;
        blocks.push(block_header);
        i += 1;
        if i % 1000 == 0 {
            println!("Processed {} blocks", i);
        }
    }

    println!("Writing file");

    let mut file = BufWriter::new(File::create("output.bin")?);

    for block in blocks.iter().rev() {
        file.write_all(block.as_ref())?;
    }

    file.flush()?;
    drop(file);

    Ok(())
}
