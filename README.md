# strftime-ruby

[![GitHub Actions](https://github.com/artichoke/strftime-ruby/workflows/CI/badge.svg)](https://github.com/artichoke/strftime-ruby/actions)
[![Code Coverage](https://codecov.artichokeruby.org/strftime-ruby/badges/flat.svg?nocache=2)](https://codecov.artichokeruby.org/strftime-ruby/index.html)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/strftime-ruby.svg)](https://crates.io/crates/strftime-ruby)
[![API](https://docs.rs/strftime-ruby/badge.svg)](https://docs.rs/strftime-ruby)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/strftime-ruby/strftime/)

`strftime-ruby` is a Ruby 3.1.2 compatible implementation of the
[`Time#strftime`] method. The `strftime` routines provided by this crate are
[POSIX-compatible], except for intentionally ignoring the `E` and `O` modified
conversion specifiers.

[`time#strftime`]: https://ruby-doc.org/core-3.1.2/Time.html#method-i-strftime
[posix-compatible]:
  https://pubs.opengroup.org/onlinepubs/9699919799/functions/strftime.html

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
strftime-ruby = "1.0.1"
```

## Crate features

All features are enabled by default.

- **std** - Enables a dependency on the Rust Standard Library. Activating this
  feature enables implementations of [`std::error::Error`] on the error types in
  this crate and the `strftime::io` module, which depends on [`std::io::Write`].
  Activating this feature also activates the **alloc** feature.
- **alloc** - Enables a dependency on the Rust [`alloc`] crate. Activating this
  feature enables the `strftime::bytes` and `stftime::string` modules, which
  depend on [`alloc::vec::Vec`] and [`alloc::string::String`]. When the
  **alloc** feature is enabled, this crate only uses [fallible allocation APIs].

[`std::error::error`]: https://doc.rust-lang.org/std/error/trait.Error.html
[`std::io::write`]: https://doc.rust-lang.org/std/io/trait.Write.html
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

This repository includes a vendored copy of [`strftime.c`] from Ruby 3.1.2,
which is licensed under the [Ruby license] or [BSD 2-clause license]. See
[`vendor/README.md`] for more details. These sources are not distributed on
[crates.io].

[`strftime.c`]: vendor/ruby-3.1.2/strftime.c
[ruby license]: vendor/ruby-3.1.2/COPYING
[bsd 2-clause license]: vendor/ruby-3.1.2/BSDL
[`vendor/readme.md`]: vendor/README.md
[crates.io]: https://crates.io/
