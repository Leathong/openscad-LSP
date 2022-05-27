openscad-LSP
========================

A [LSP](https://microsoft.github.io/language-server-protocol/) (Language Server Protocol) 
server for [OpenSCAD](https://openscad.org). 

inspired by [dzhu/openscad-language-server](https://github.com/dzhu/openscad-language-server)

Tested with VSCode on Mac and Windows. [[vscode extension]](https://github.com/Leathong/openscad-support-vscode)

Tested with lsp-mode on Emacs on Linux by [@Lenbok](https://github.com/Lenbok).

Features
--------

-   code and path auto-completion
-   jump to definition
-   code snippets
-   function/module signatures on hover
-   document symbols
-   formatter, utilizing clang-format, you need install it youself, it is not built-in.
-   hover and suggestion documentation, read from comments before the function/module.</br>
    If you want to write documentation for the [builtin](src/builtins.scad) function/module, feel free to submit pr.

Build
------------

openscad-LSP is written in [Rust](https://rust-lang.org), in order to use it, you need to
install [Rust toolchain](https://www.rust-lang.org/learn/get-started).


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
        --fmt-exe <FMT_EXE>        clang format executable file path [default: clang-format]
        --fmt-style <FMT_STYLE>    LLVM, GNU, Google, Chromium, Microsoft, Mozilla, WebKit, file
                                   [default: Microsoft]
    -h, --help                     Print help information
        --ip <IP>                  [default: 127.0.0.1]
    -p, --port <PORT>              [default: 3245]
    -V, --version                  Print version information
```

If you work with vscode, you can install the extension directly form the [marketplace](https://marketplace.visualstudio.com/items?itemName=Leathong.openscad-language-support&ssr=false#overview)
