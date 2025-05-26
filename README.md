> **NOTICE: Because clang-format produces poor formatting results for OpenSCAD, we have completely removed support for clang-format and switched to the new formatter [topiary](https://github.com/tweag/topiary)**

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
-   formatter, utilizing topiary.
-   variable / module renaming
-   hover and suggestion documentation, read from comments before the function/module.</br>


IDE plugins
--------

| IDE | Plugin | Note  |
| --- | ------ | ----- |
| Neovim  | [mason.nvim](https://github.com/williamboman/mason.nvim)    | Only tested on Mac and Linux     |
| Neovim  | [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)  | Only tested on Mac and Linux     |
| VS Code | [openscad-language-support](https://github.com/Leathong/openscad-support-vscode)  | Only tested on Mac and Windows   |
| Emacs   | [lsp-bridge](https://github.com/manateelazycat/lsp-bridge)  | Only tested on Mac and Linux   |


Install
------------

openscad-LSP is written in [Rust](https://rust-lang.org), in order to use it, you need to
install [Rust toolchain](https://www.rust-lang.org/learn/get-started).

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
  -p, --port <PORT>              [default: 3245]
      --ip <IP>                  [default: 127.0.0.1]
      --builtin <BUILTIN>        external builtin functions file path, if set, the built-in builtin functions file will not be used [default: ]
      --stdio                    use stdio instead of tcp
      --ignore-default           exclude default params in auto-completion
      --depth <DEPTH>            search depth [default: 3]
      --indent <INDENT>          The indentation string used for that particular language. Defaults to "  " if not provided. Any string can be provided, but in most instances will be some whitespace: "  ", "    ", or "\t"
      --query-file <QUERY_FILE>  The query file used for topiary formatting
  -h, --help                     Print help
  -V, --version                  Print version
```

To change the config at runtime, you can send notification `workspace/didChangeConfiguration`

```jsonc
// example
{
  "settings": {
    "openscad": {
      "search_paths": "/libs",
      "indent": "    ",
      "query_file": "path/to/my/openscad.scm",
      "default_param": true
    }
  }
}
```
