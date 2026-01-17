# nyaa-rs

a lightweight tui app to search nyaa.si

## features

- sort search results
- theme support with hot reloading
- auto opens your torrent client to download

## install

```bash
git clone https://github.com/offs/nyaa-rs
cd nyaa-rs
cargo install --path .
```

## config

theme: `~/.config/nyaa/theme.json`
- or on windows `theme.json` in the same directory as the binary

```json
{
  "fg": "#ebdbb2",
  "primary": "#fe8019",
  "secondary": "#83a598",
  "selection_bg": "#3c3836",
  "border": "#504945",
  "border_focus": "#fe8019"
}
```
