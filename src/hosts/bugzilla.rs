use crate::{hosts::UA, Error, Host, Info, State};
use reqwest::{header::USER_AGENT, Client};
use serde::Deserialize;
use url::Url;
use url_matcher::FromPattern;

#[derive(Deserialize)]
struct PatternData {
    number: String,
}
impl FromPattern<Self> for PatternData {}

#[derive(Deserialize)]
struct Response {
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

    let url = format!("https://{domain}/rest/bug/{}", data.number);

    let mut request = client.get(url);
    request = request.header(USER_AGENT, UA);

    // TODO: not implemented
    // if let Ok(token) = std::env::var(host.get_token_var()) {
    //     request = request.bearer_auth(token);
    // }

    let response = request.send().await?.error_for_status()?;
    let resp: Response = response.json().await?;

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
