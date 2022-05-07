openscad-LSP
========================

A [LSP](https://microsoft.github.io/language-server-protocol/) (Language Server Protocol) 
server for [OpenSCAD](https://openscad.org). 

inspired by [dzhu/openscad-language-server](https://github.com/dzhu/openscad-language-server)

Tested with VSCode on Mac and Windows.

Features
--------

-   code and path auto-completion
-   jump to definition
-   code snippets
-   function/module signatures on hover
-   document symbols

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

If you work with vscode, you can install the extension directly form the [marketplace](https://marketplace.visualstudio.com/items?itemName=Leathong.openscad-language-support&ssr=false#overview)
