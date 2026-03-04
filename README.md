# koda-agent (Deprecated)

> **⚠️ This crate has been deprecated.**
> Koda has been refactored into two new crates for better modularity:
>
> - **[`koda-core`](https://crates.io/crates/koda-core)** — engine library
> - **[`koda-cli`](https://crates.io/crates/koda-cli)** — CLI binary (installs the `koda` command)
>
> Development continues at [github.com/lijunzh/koda](https://github.com/lijunzh/koda).

## Migrating

```bash
cargo install koda-cli
```

This installs the same `koda` binary, replacing the one from `koda-agent`.
All your existing config (`~/.config/koda/`) and project data (`.koda.db`) carry over — no migration needed.

## History

`koda-agent` was the original single-crate implementation of Koda, a high-performance AI coding agent built in Rust. Versions v0.1.0 through v0.1.4 contained the full agent. v0.1.5 is the final release, adding a deprecation notice pointing users to `koda-cli`.

For the full changelog, see [CHANGELOG.md](CHANGELOG.md).

## License

MIT
