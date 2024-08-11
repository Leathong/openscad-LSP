openscad-LSP
========================

A [LSP](https://microsoft.github.io/language-server-protocol/) (Language Server Protocol)
server for [OpenSCAD](https://openscad.org).

inspired by [dzhu/openscad-language-server](https://github.com/dzhu/openscad-language-server)

Tested with VSCode on Mac and Windows. [[vscode extension]](https://github.com/Leathong/openscad-support-vscode)

Tested with lsp-mode on Emacs on Linux by [@Lenbok](https://github.com/Lenbok).

Features
--------

-   builtin function/module documents
-   code and path auto-completion
-   jump to definition
-   code snippets
-   function/module signatures on hover
-   document symbols
-   formatter, utilizing clang-format, you need install it yourself, it is not built-in.
-   variable / module renaming
-   hover and suggestion documentation, read from comments before the function/module.</br>


IDE plugins
--------

| IDE | Plugin | Note  |
| --- | ------ | ----- |
| Neovim  | [mason.nvim](https://github.com/williamboman/mason.nvim)    | Only tested on Mac and Linux     |
| Neovim  | [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)  | Only tested on Mac and Linux     |
| VS Code | [openscad-language-support](https://github.com/Leathong/openscad-support-vscode)  | Only tested on Mac and Windows   |


Install
------------

openscad-LSP is written in [Rust](https://rust-lang.org), in order to use it, you need to
install [Rust toolchain](https://www.rust-lang.org/learn/get-started).

To use `scadforamt`, download the binary from the repo's [releases page](https://github.com/hugheaves/scadformat/releases)

``` {.sh}
cargo install openscad-lsp
```

Build
------------

``` {.sh}
cd openscad-LSP
cargo build --release
```

Usage
-----

The server communicates over TCP socket (127.0.0.1:3245).

```
A language(LSP) server for OpenSCAD

Usage: openscad-lsp [OPTIONS]

Options:
  -p, --port <PORT>            [default: 3245]
      --ip <IP>                [default: 127.0.0.1]
      --fmt-style <FMT_STYLE>  formatting style for clang-format [possible values: LLVM, GNU, Google, Chromium, Microsoft, Mozilla, Webkit, file]
      --fmt-exe <FMT_EXE>      formatter executable file path [default: clang-format]
      --fmt-args <FMT_ARGS>    formatter executable arguments
      --builtin <BUILTIN>      external builtin functions file path, if set, the built-in builtin functions file will not be used
      --stdio                  use stdio instead of tcp
      --ignore-default         exclude default params in auto-completion
      --depth <DEPTH>          search depth [default: 3]
  -h, --help                   Print help
  -V, --version                Print version
```

To change the config during running, you can send notification `workspace/didChangeConfiguration`

`clang-format` formatting config:

```js
// example
{
    "settings": {
        "openscad": {
            "search_paths": "/libs",
            "fmt_exe": "/usr/bin/clang-format",
            "fmt_style": "file",
            "default_param": true
        }
    }
}
```

`scadformat` formatting config:

```js
{
    "settings": {
        "openscad": {
            "search_paths": "/libs",
            "fmt_exe": "scadformat",
            "fmt_args": "--log-level error",
            "default_param": true
        }
    }
}
```

To format on write with neovim, include the text below in your `init.lua`:
```lua
autocmd("BufWritePre", {
  pattern = {"*.scad" },
  callback = function() vim.lsp.buf.format() end,
})
```
