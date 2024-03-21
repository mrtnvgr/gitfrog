mod hosts;

pub use hosts::Host;
use url::Url;

#[derive(Debug, Eq, PartialEq)]
pub struct Info {
    pub title: String,
    pub state: State,
}

impl Info {
    pub async fn from_url(url: &Url) -> Result<Self, Error> {
        let domain = url.domain().ok_or(Error::InvalidURL)?;
        Host::from_domain(domain)?.get(url).await
    }

    pub async fn from_urls(urls: &Vec<Url>) -> Vec<Result<Self, Error>> {
        let mut tasks = Vec::new();

        for url in urls {
            tasks.push(Self::from_url(url));
        }

        futures::future::join_all(tasks).await
    }

    pub async fn from_urls_ref(urls: Vec<&Url>) -> Vec<Result<Self, Error>> {
        let mut tasks = Vec::new();

        for url in urls {
            tasks.push(Self::from_url(url));
        }

        futures::future::join_all(tasks).await
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum State {
    Open,
    Closed,
    Merged,
    Draft,
}

impl State {
    pub const fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    pub const fn as_str(&self) -> &str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Merged => "merged",
            Self::Draft => "draft",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not parse given url")]
    InvalidURL,
    #[error("Failed to auto-detect host: \"{0}\"")]
    UnknownHost(String),
    #[error("Failed to match the link")]
    FailedMatch,
    #[error("Failed to pattern match received state: \"{0}\"")]
    UnknownState(String),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    URLMatcherError(#[from] url_matcher::Error),
}

#[cfg(test)]
mod tests;
