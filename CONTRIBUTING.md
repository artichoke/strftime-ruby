# Contributing to Artichoke – strftime-ruby

Welcome to [Artichoke]. Thanks for taking the time to contribute.

> [!NOTE]  
> This crate is feature complete and frozen. It will not receive further
> development.

`strftime-ruby` implements Ruby-compatible `Time#strftime` formatting. If its
formatting differs from Ruby or POSIX behavior, please [file an issue].

## Setup

This repository uses [mise] to manage Rust, Node.js, and repository development
tools. Install the declared toolchains and text-formatting dependencies with:

```sh
mise install
mise run pnpm-install
```

Run `mise tasks` to list the available development commands.

## Validation

Format, lint, and test changes with:

```sh
mise run fmt
mise run lint
mise run test
```

Pull requests must include appropriate tests and pass all required checks before
merging.

## Publishing

Maintainers publish releases through crates.io trusted publishing. See
[`docs/publishing.md`](docs/publishing.md) for the trust configuration, release
procedure, and failure-recovery guidance.

## Updating dependencies

Regular dependency updates are handled by [Dependabot]. Keep dependency changes
focused and run the full validation suite before merging them.

[artichoke]: https://github.com/artichoke
[dependabot]: https://docs.github.com/code-security/dependabot
[file an issue]: https://github.com/artichoke/strftime-ruby/issues/new
[mise]: https://mise.jdx.dev/
