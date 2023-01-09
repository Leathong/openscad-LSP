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
-   hover and suggestion documentation, read from comments before the function/module.</br>


IDE plugins
--------

| IDE | Plugin | Note  |
| --- | ------ | ----- |
| Neovim  | [mason.nvim](https://github.com/williamboman/mason.nvim)    | Only tested on Mac and Linux     |
| Neovim  | [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)  | Only tested on Mac and Linux     |
| VS Code | [openscad-language-support](https://marketplace.visualstudio.com/items?itemName=Leathong.openscad-language-support)  | Only tested on Mac and Windows   |


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
USAGE:
    openscad-lsp [OPTIONS]

OPTIONS:
        --builtin <BUILTIN>        external builtin functions file path, if set, the built-in
                                   builtin functions file will not be used [default: ]
        --fmt-exe <FMT_EXE>        clang format executable file path [default: clang-format]
        --fmt-style <FMT_STYLE>    LLVM, GNU, Google, Chromium, Microsoft, Mozilla, WebKit, file
                                   [default: Microsoft]
    -h, --help                     Print help information
        --ignore-default           exclude default params in auto-completion
        --ip <IP>                  [default: 127.0.0.1]
    -p, --port <PORT>              [default: 3245]
        --stdio                    use stdio instead of tcp
    -V, --version                  Print version information
```

To change the config during running, you can send notification `workspace/didChangeConfiguration` 

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
