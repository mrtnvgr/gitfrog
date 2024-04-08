use crate::{hosts::UA, Error, Host, Info, State};
use reqwest::{header::USER_AGENT, Client, Response};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;
use url_matcher::FromPattern;

#[derive(Deserialize)]
struct PatternData {
    number: String,
}
impl FromPattern<Self> for PatternData {}

#[derive(Deserialize)]
struct ZillaResponse {
    bugs: Vec<Bug>,
}

#[derive(Deserialize)]
struct Bug {
    summary: String,
    is_open: bool,
}

pub async fn get(host: &Host<'_>, domain: &str, url: &Url) -> Result<Vec<Info>, Error> {
    let client = Client::new();

    let query = url.query().unwrap_or_default();
    let path = url.path();
    let data = format!("{path}?{query}");

    let data = PatternData::from_pattern(host.pattern(), &data)?;

    let url = match host {
        Host::Bugzilla(_) => get_rest_url(domain, &data),
        Host::BugzillaJsonRpc(_) => get_jsonrpc_url(domain, &data)?,
        _ => unreachable!(),
    };

    let mut request = client.get(url);
    request = request.header(USER_AGENT, UA);

    // TODO: not implemented
    // if let Ok(token) = std::env::var(host.get_token_var()) {
    //     request = request.bearer_auth(token);
    // }

    let response = request.send().await?.error_for_status()?;

    let resp: ZillaResponse = match host {
        Host::Bugzilla(_) => response.json().await?,
        Host::BugzillaJsonRpc(_) => get_jsonrpc_resp(response).await?,
        _ => unreachable!(),
    };

    let mut infos = Vec::new();

    for bug in resp.bugs {
        let title = bug.summary;

        // TODO: add support for: resolved, wontfix
        let state = if bug.is_open {
            State::Open
        } else {
            State::Closed
        };

        infos.push(Info { title, state });
    }

    Ok(infos)
}

async fn get_jsonrpc_resp(response: Response) -> Result<ZillaResponse, Error> {
    let json: HashMap<String, Value> = response.json().await?;
    let result = json.get("result").ok_or(Error::InvalidURL)?;
    Ok(serde_json::from_value(result.clone())?)
}

fn get_rest_url(domain: &str, data: &PatternData) -> String {
    format!("https://{domain}/rest/bug/{}", data.number)
}

fn get_jsonrpc_url(domain: &str, data: &PatternData) -> Result<String, Error> {
    let base = format!("https://{domain}/jsonrpc.cgi");
    let params = format!("[{{\"ids\": [{}]}}]", data.number);
    let iter = vec![("method", "Bug.get"), ("params", &params)];
    let url = Url::parse_with_params(&base, iter).map_err(|_| Error::InvalidURL)?;
    Ok(url.to_string())
}
