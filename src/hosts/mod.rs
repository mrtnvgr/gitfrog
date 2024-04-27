use url::Url;

mod bugzilla;
mod git;

use crate::{Error, Info};

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:110.0) Gecko/20100101 Firefox/110.0";

const GITHUB_DOMAIN: &str = "github.com";
const GITLAB_DOMAIN: &str = "gitlab.com";
const CODEBERG_DOMAIN: &str = "codeberg.org";
const FREEDESKTOP_DOMAIN: &str = "gitlab.freedesktop.org";
const WINEHQ_DOMAIN: &str = "bugs.winehq.org";

pub enum Host<'a> {
    Github,
    Gitlab(&'a str),
    /// Also compatible with Codeberg and Forgejo instances
    Gitea(&'a str),
    Bugzilla(&'a str),
    /// Useful for 4.x instances of bugzilla
    BugzillaJsonRpc(&'a str),
}

impl<'a> Host<'a> {
    pub async fn get(&self, url: &Url) -> Result<Info, Error> {
        log::debug!("Getting {url}");
        match self {
            Self::Github | Self::Gitlab(_) | Self::Gitea(_) => git::get(self, url).await,
            // TODO: optimize for `from_urls`
            Self::Bugzilla(domain) | Self::BugzillaJsonRpc(domain) => {
                bugzilla::get(self, domain, url)
                    .await
                    .and_then(|x| x.first().cloned().ok_or(Error::Unreachable))
            }
        }
    }

    pub(crate) fn from_domain(domain: &'a str) -> Result<Self, Error> {
        match domain {
            GITHUB_DOMAIN => Ok(Self::Github),
            CODEBERG_DOMAIN => Ok(Self::Gitea(domain)),
            GITLAB_DOMAIN | FREEDESKTOP_DOMAIN => Ok(Self::Gitlab(domain)),
            WINEHQ_DOMAIN => Ok(Self::BugzillaJsonRpc(domain)),
            _ => Err(Error::UnknownHost(domain.to_owned())),
        }
    }

    fn get_token_var(&self) -> String {
        match self {
            Host::Github => String::from("GITHUB_TOKEN"),
            Host::Gitlab(GITLAB_DOMAIN) => String::from("GITLAB_TOKEN"),
            Host::Gitea(CODEBERG_DOMAIN) => String::from("CODEBERG_TOKEN"),
            Host::Gitlab(x) | Host::Gitea(x) => {
                format!("{}_TOKEN", x.replace('.', "").to_ascii_uppercase())
            }
            Host::Bugzilla(_) | Host::BugzillaJsonRpc(_) => todo!("not implemented"),
        }
    }

    const fn pattern(&self) -> &'static str {
        match &self {
            Self::Github | Self::Gitea(_) => "/:owner/:repo/:kind/:number",
            Self::Gitlab(_) => "/:owner/:repo/-/:kind/:number",
            Self::Bugzilla(_) | Self::BugzillaJsonRpc(_) => "/show_bug.cgi?id=:number",
        }
    }
}
