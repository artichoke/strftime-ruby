# strftime-ruby

[![GitHub Actions](https://github.com/artichoke/strftime-ruby/workflows/CI/badge.svg)](https://github.com/artichoke/strftime-ruby/actions)
[![Code Coverage](https://codecov.artichokeruby.org/strftime-ruby/badges/flat.svg)](https://codecov.artichokeruby.org/strftime-ruby/index.html)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/strftime-ruby.svg)](https://crates.io/crates/strftime-ruby)
[![API](https://docs.rs/strftime-ruby/badge.svg)](https://docs.rs/strftime-ruby)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/strftime-ruby/strftime/)

`strftime` parser and formatter. Used to implement [`Time#strftime`] from the
Ruby Core library in [Artichoke Ruby][artichoke].

[`time#strftime`]: https://ruby-doc.org/core-3.1.2/Time.html#method-i-strftime
[artichoke]: https://github.com/artichoke/artichoke

> Formats time according to the directives in the given format string.
>
> The directives begin with a percent (%) character. Any text not listed as a
> directive will be passed through to the output string.
>
> The directive consists of a percent (%) character, zero or more flags,
> optional minimum field width, optional modifier and a conversion specifier as
> follows:
>
> ```text
> %<flags><width><modifier><conversion>
> ```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
strftime-ruby = "0.1.0"
```

## Crate features

All features are enabled by default.

- **std** - Enables a dependency on the Rust Standard Library. Activating this
  feature enables implementations of [`std::error::Error`] on the error types in
  this crate. Activating this feature also activates the **alloc** feature.
- **alloc** - Enables a dependency on the Rust [`alloc`] crate. Activating this
  feature enables APIs that require [`alloc::vec::Vec`] or
  [`alloc::string::String`]. When the **alloc** feature is enabled, this crate
  only uses [fallible allocation APIs].

[`std::error::error`]: https://doc.rust-lang.org/std/error/trait.Error.html
[`alloc`]: https://doc.rust-lang.org/alloc/
[`alloc::vec::vec`]: https://doc.rust-lang.org/alloc/vec/struct.Vec.html
[`alloc::string::string`]:
  https://doc.rust-lang.org/alloc/string/struct.String.html
[fallible allocation apis]:
  https://doc.rust-lang.org/alloc/vec/struct.Vec.html#method.try_reserve

### Minimum Supported Rust Version

This crate requires at least Rust 1.58.0. This version can be bumped in minor
releases.

## License

`strftime-ruby` is licensed under the [MIT License](LICENSE) (c) Ryan Lopopolo
and x-hgg-x.
