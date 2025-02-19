
# `uf`

`uf` is a minimalistic file opener written in Rust.

It features:
- An extremely simple and yet powerful configuration file format
- A tiny codebase entirely written in safe Rust
- No dependencies except for the Rust standard library and
  [anyhow](https://crates.io/crates/anyhow) for error handling

## Usage

`uf` is configured using a file located at `~/.config/uf.conf`.
The configuration file is a text file where every line defines a rule.

The following example configuration file illustrates the syntax:
```plaintext
# We can configure zathura as PDF viewer:
mime application/pdf zathura

# Alternatively, you can also create rules based on file extensions:
ext epub zathura

# If there are multiple rules that match a file, the first one is used.
# This way, we could configure uf to open C source files with emacs...
mime text/x-c emacs
# ...and to open all other text files with vim.
# Note the use of a wildcard in the MIME type to match all subtypes.
mime text/* vim
```

After you have configured `uf`, opening a file is as simple as running
```sh
uf <file>
```

## Installation

To install `uf`, you need to have `cargo` installed.
If you don't have `cargo` installed, you can install it by following the
instructions at
<https://doc.rust-lang.org/cargo/getting-started/installation.html>.

After you have `cargo` installed, you can install `uf` by running the
following command:

```sh
cargo install uf
```

## License

```plaintext
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <https://unlicense.org/>
```

