# gitfrog

[![Tests](https://github.com/mrtnvgr/gitfrog/actions/workflows/rust.yml/badge.svg)](https://github.com/mrtnvgr/gitfrog/actions/workflows/rust.yml)

Get current info about PRs and issues

## Supported

- Github, Gitlab, Gitea
- Bugzilla

## Examples

### Auto-detection for well-known forges

```rust
use gitfrog::Info;

let url = Url::parse("https://github.com/catppuccin/nvim/pull/8").unwrap();
let info = Info::from_url(&url).await.unwrap();

assert_eq!(info.title, "Add Kitty themes");
assert_eq!(info.state, State::Closed);
assert_eq!(info.state.is_open(), false);
```

> [!NOTE]
> Feel free to file an issue about missing forges. Contributions are welcome too!

### Self-hosted instances

```rust
use gitfrog::Host;

let url = Url::parse("https://gitlab.freedesktop.org/wayland/wayland/-/issues/369").unwrap();
let domain = url.domain().unwrap();
let info = Host::Gitlab(domain).get(&url).await.unwrap();

assert_eq!(info.title, "libwayland-server.so.0.21.0 ends in segfault and then somehow shuts down my system.");
assert_eq!(info.state, State::Closed);
assert_eq!(info.state.is_open(), false);
```

### Multiple urls in parallel

```rust
let wrapper = |x| Url::parse(&format!("https://github.com/catppuccin/nvim/pull/{x}")).unwrap();
let urls: Vec<Url> = vec![110, 143, 1].iter().map(wrapper).collect();

let info = Info::from_urls(&urls).await;

assert_eq!(info[0].as_ref().unwrap().title, "fix: typo");
assert_eq!(info[1].as_ref().unwrap().title, "fix: #103");
assert_eq!(info[2].is_err(), true);
```
