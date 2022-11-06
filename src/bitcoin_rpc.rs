use std::path::PathBuf;

use crate::jsonrpc::make_request;
use anyhow::bail;
use reqwest::{Client, RequestBuilder};

const BLOCK_HEADER_SIZE: usize = 80;
type BlockHeader = Box<[u8; BLOCK_HEADER_SIZE]>;

struct BitcoinCookie {
    username: String,
    password: String,
}

fn load_bitcoin_cookie() -> anyhow::Result<BitcoinCookie> {
    let path = PathBuf::from(std::env::var("HOME")?).join(".bitcoin/.cookie");

    if !path.exists() {
        bail!("Can't find bitcoin cookie");
    }

    let cookie = std::fs::read_to_string(path)?;
    let split: Vec<&str> = cookie.trim().split(':').collect();
    if split.len() != 2 {
        bail!("Bad cookie file");
    }

    Ok(BitcoinCookie {
        username: split[0].to_owned(),
        password: split[1].to_owned(),
    })
}

pub struct BitcoinRpc {
    client: Client,
    cookie: BitcoinCookie,
}

impl BitcoinRpc {
    pub fn new_localhost() -> anyhow::Result<Self> {
        let cookie = load_bitcoin_cookie()?;
        Ok(BitcoinRpc {
            client: Client::new(),
            cookie,
        })
    }

    pub fn create_builder(&self) -> RequestBuilder {
        const LOCALHOST_URL: &str = "http://localhost:8332/";
        self.client
            .post(LOCALHOST_URL)
            .basic_auth(&self.cookie.username, Some(&self.cookie.password))
    }

    pub async fn get_best_block_hash(&self) -> anyhow::Result<String> {
        let empty: [(); 0] = [];
        let response: Option<String> = make_request(
            &self.client,
            self.create_builder(),
            "getbestblockhash",
            &empty,
        )
        .await?;
        Ok(response.unwrap())
    }

    pub async fn get_binary_block_header(
        &self,
        block_hash: &str,
    ) -> anyhow::Result<Option<BlockHeader>> {
        let header_opt: Option<String> = make_request(
            &self.client,
            self.create_builder(),
            "getblockheader",
            &(block_hash, false),
        )
        .await?;

        let Some(header) = header_opt else  {
            return Ok(None);
        };

        let decoded = hex::decode(header)?;
        let boxed_slice = decoded.into_boxed_slice();
        let boxed_array =
            BlockHeader::try_from(boxed_slice).map_err(|_| anyhow::anyhow!("Unexpected size"))?;
        Ok(Some(boxed_array))
    }
}
