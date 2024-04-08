use reqwest::{header::USER_AGENT, Client};
use serde::Deserialize;
use serde_json::Value as JSONValue;
use std::collections::HashMap;
use url::{form_urlencoded, Url};
use url_matcher::FromPattern;

use super::UA;
use crate::hosts::GITHUB_DOMAIN;
use crate::{Error, Host, Info, State};

#[derive(Deserialize)]
struct PatternData {
    owner: String,
    repo: String,
    kind: String,
    number: String,
}
impl FromPattern<Self> for PatternData {}

pub async fn get(host: &Host<'_>, url: &Url) -> Result<Info, Error> {
    let client = Client::new();
    let data = PatternData::from_pattern(host.pattern(), url.path())?;

    let url = get_api_link(host, &data);

    let mut request = client.get(url);
    request = request.header(USER_AGENT, UA);

    if let Ok(token) = std::env::var(host.get_token_var()) {
        request = request.bearer_auth(token);
    }

    let response = request.send().await?.error_for_status()?;
    let data: HashMap<String, JSONValue> = response.json().await?;

    let (title, state) = get_title_state(&data)?;

    Ok(Info { title, state })
}

#[rustfmt::skip]
fn get_api_link(host: &Host<'_>, data: &PatternData) -> String {
    let owner = &data.owner;
    let repo = &data.repo;
    let kind = &data.kind;
    let number = &data.number;

    let kind = match (host, kind.as_str()) {
        (Host::Github, "pull") => "pulls",
        (_, x) => x,
    };

    let owner_repo = format!("{owner}/{repo}");
    let encoded: String = form_urlencoded::byte_serialize(owner_repo.as_bytes()).collect();

    match host {
        Host::Github => format!("https://api.{GITHUB_DOMAIN}/repos/{owner}/{repo}/{kind}/{number}"),
        Host::Gitlab(domain) => format!("https://{domain}/api/v4/projects/{encoded}/{kind}/{number}"),
        Host::Gitea(domain) => format!("https://{domain}/api/v1/repos/{owner}/{repo}/{kind}/{number}"),
        _ => unreachable!(),
    }
}

fn get_title_state(data: &HashMap<String, JSONValue>) -> Result<(String, State), Error> {
    let title = data
        .get("title")
        .and_then(|x| x.as_str())
        .ok_or(Error::FailedMatch)?
        .to_owned();

    let state = data
        .get("state")
        .and_then(|x| x.as_str())
        .ok_or(Error::FailedMatch)?;

    let mut state = match state {
        "open" | "opened" => State::Open,
        "closed" | "locked" => State::Closed,
        "merged" => State::Merged,
        _ => return Err(Error::UnknownState(state.to_owned())),
    };

    // Github, Gitea
    if let Some(merged) = data.get("merged").and_then(JSONValue::as_bool) {
        state = if merged { State::Merged } else { state };
    }

    // Github
    if let Some(draft) = data.get("draft").and_then(JSONValue::as_bool) {
        state = if draft { State::Draft } else { state };
    }

    // Gitea
    let wip_prefixes = ["wip:", "[wip]"];
    if wip_prefixes.iter().any(|prefix| title.starts_with(prefix)) {
        state = State::Draft;
    }

    Ok((title, state))
}
