use anyhow::bail;
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct Request<'a, P: Serialize> {
    method: &'a str,
    params: P,
    id: &'a str,
}

#[derive(Deserialize)]
struct Response<R> {
    error: Option<Value>,
    result: Option<R>,
}

pub async fn make_request<P: Serialize, R: DeserializeOwned>(
    client: &Client,
    request_builder: RequestBuilder,
    method: &str,
    params: P,
) -> anyhow::Result<Option<R>> {
    let id = "1";
    let jsonrpc_request = Request { id, method, params };

    let http_request = request_builder.json(&jsonrpc_request).build()?;
    let http_response = client.execute(http_request).await?;
    if !http_response.status().is_success() {
        println!("{:?}", http_response.text().await?);
        bail!("bad");
    }
    let response = http_response.json::<Response<R>>().await?;

    if let Some(error) = response.error {
        bail!("An error occurred {}", error);
    }

    Ok(response.result)
}
