use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum DomainRecordResponse {
    Ok { domain_record: DomainRecord },
    Error { id: String, message: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum DomainRecordsResponse {
    Ok {
        domain_records: Vec<DomainRecord>,

        links: serde_json::Value,

        meta: serde_json::Value,
    },
    Error {
        id: String,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DomainRecord {
    pub id: i64,
    pub name: String,
    pub data: String,
    pub ttl: i32,

    #[serde(rename = "type")]
    pub kind: String,
}

pub struct ApiClient {
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            "application/json"
                .parse()
                .expect("Faile to set default Accept header"),
        );
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {token}")
                .parse()
                .expect("Failed to set default Authorization header"),
        );
        Self {
            http: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn get_records(
        &self,
        domain: &str,
        per_page: Option<u16>,
        kind: Option<&str>,
        name: Option<&str>,
    ) -> Result<Vec<DomainRecord>> {
        let mut url = format!("https://api.digitalocean.com/v2/domains/{domain}/records");

        let mut params = Vec::new();
        if let Some(per_page) = per_page {
            params.push(format!("per_page={per_page}"));
        }
        if let Some(kind) = kind {
            params.push(format!("type={kind}"));
        }
        if let Some(name) = name {
            params.push(format!("name={name}"));
        }
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let response = self
            .http
            .get(url)
            .send()
            .await
            .context("Failed to send GET request (get_records)")?
            .text()
            .await
            .context("Failed to fetch GET response (get_records)")?;
        log::debug!("get_records: {response}");

        let response = serde_json::from_str(&response)
            .context("Failed to parse GET response (get_records)")?;
        match response {
            DomainRecordsResponse::Ok { domain_records, .. } => Ok(domain_records),
            DomainRecordsResponse::Error { id, message } => bail!("{}: {}", id, message),
        }
    }

    pub async fn update_record(
        &self,
        domain: &str,
        id: i64,
        kind: &str,
        name: &str,
        data: &str,
        ttl: u32,
    ) -> Result<DomainRecord> {
        let response = self
            .http
            .put(format!(
                "https://api.digitalocean.com/v2/domains/{domain}/records/{id}"
            ))
            .json(&serde_json::json!({
                "name": name,
                "type": kind,
                "data": data,
                "ttl": ttl,
            }))
            .send()
            .await
            .context("Failed to send PUT request (update_record)")?
            .text()
            .await
            .context("Failed to fetch PUT response (update_record)")?;
        log::debug!("update_record: {response}");

        let response = serde_json::from_str(&response)
            .context("Failed to parse PUT response (create_record)")?;
        match response {
            DomainRecordResponse::Ok { domain_record } => Ok(domain_record),
            DomainRecordResponse::Error { id, message } => bail!("{}: {}", id, message),
        }
    }

    pub async fn create_record(
        &self,
        domain: &str,
        name: &str,
        kind: &str,
        data: &str,
        ttl: u32,
    ) -> Result<DomainRecord> {
        let response = self
            .http
            .post(format!(
                "https://api.digitalocean.com/v2/domains/{domain}/records/"
            ))
            .json(&serde_json::json!({
                "name": name,
                "type": kind,
                "data": data,
                "ttl": ttl,
            }))
            .send()
            .await
            .context("Failed to send POST request (create_record)")?
            .text()
            .await
            .context("Failed to fetch POST response (create_record)")?;
        log::debug!("create_record: {response}");

        let response = serde_json::from_str(&response)
            .context("Failed to parse POST response (create_record)")?;
        match response {
            DomainRecordResponse::Ok { domain_record } => Ok(domain_record),
            DomainRecordResponse::Error { id, message } => bail!("{}: {}", id, message),
        }
    }
}
