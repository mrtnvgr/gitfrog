use super::*;

#[tokio::test]
async fn github() {
    let url = Url::parse("https://github.com/catppuccin/nvim/pull/8").unwrap();
    let info = Info::from_url(&url).await.unwrap();

    assert_eq!(info.title, "Add Kitty themes");
    assert_eq!(info.state, State::Merged);
    assert!(!info.state.is_open());
}

#[tokio::test]
async fn gitlab() {
    let url = Url::parse(
        "https://gitlab.com/simple-nixos-mailserver/nixos-mailserver/-/merge_requests/319",
    )
    .unwrap();
    let info = Info::from_url(&url).await.unwrap();

    assert_eq!(info.title, "dovecot: support new `sieve` API in nixpkgs");
    assert_eq!(info.state, State::Merged);
    assert!(!info.state.is_open());

    let url =
        Url::parse("https://gitlab.com/simple-nixos-mailserver/nixos-mailserver/-/issues/279")
            .unwrap();
    let info2 = Info::from_url(&url).await.unwrap();

    assert_eq!(info2.title, "OpenDKIM rights problem");
    assert_eq!(info2.state, State::Open);
    assert!(info2.state.is_open());
}

#[tokio::test]
async fn gitlab_custom() {
    let url = Url::parse("https://gitlab.freedesktop.org/wayland/wayland/-/issues/369").unwrap();
    let domain = url.domain().unwrap();
    let status = Host::Gitlab(domain).get(&url).await.unwrap();

    assert_eq!(
        status.title,
        "libwayland-server.so.0.21.0 ends in segfault and then somehow shuts down my system."
    );
    assert_eq!(status.state, State::Closed);
    assert!(!status.state.is_open());
}

#[tokio::test]
async fn gitea() {
    let url = Url::parse("https://codeberg.org/dnkl/foot/issues/1642").unwrap();
    let info = Info::from_url(&url).await.unwrap();

    assert_eq!(info.title, "Kitty keyboard protocol broken?");
    assert_eq!(info.state, State::Closed);
    assert!(!info.state.is_open());

    let url = Url::parse("https://codeberg.org/dnkl/foot/pulls/1640").unwrap();
    let info = Info::from_url(&url).await.unwrap();

    assert_eq!(
        info.title,
        "sixel: trim trailing, fully transparent sixel rows"
    );
    assert_eq!(info.state, State::Merged);
    assert!(!info.state.is_open());
}

#[tokio::test]
async fn gitea_custom() {
    let url = Url::parse("https://projects.blender.org/blender/blender/issues/35100").unwrap();
    let domain = url.domain().unwrap();
    let info = Host::Gitea(domain).get(&url).await.unwrap();

    assert_eq!(
        info.title,
        "Dynamic Sculpting: Inflate Brush artifacts --- could freeze blender"
    );
    assert_eq!(info.state, State::Open);
    assert!(info.state.is_open());
}

#[tokio::test]
async fn from_urls() {
    let wrapper = |x| Url::parse(&format!("https://github.com/catppuccin/nvim/pull/{x}")).unwrap();
    let urls: Vec<Url> = [110, 143, 1].iter().map(wrapper).collect();

    let info = Info::from_urls(&urls).await;

    assert_eq!(info[0].as_ref().unwrap().title, "fix: typo");
    assert_eq!(info[1].as_ref().unwrap().title, "fix: #103");
    assert!(info[2].is_err());
}

#[tokio::test]
async fn bugzilla_rpc() {
    let url = Url::parse("https://bugs.winehq.org/show_bug.cgi?id=54692").unwrap();
    let info = Info::from_url(&url).await.unwrap();

    assert_eq!(
        info.title,
        "Many DX11 applications crashes after applying wined3d-bindless-texture patch"
    );
    assert_eq!(info.state, State::Open);
}
