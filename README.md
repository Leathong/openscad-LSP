openscad-LSP
========================

A [LSP](https://microsoft.github.io/language-server-protocol/) (Language Server Protocol) 
server for [OpenSCAD](https://openscad.org). 

inspired by [dzhu/openscad-language-server](https://github.com/dzhu/openscad-language-server)

Tested with VSCode on Mac and Windows. [[vscode extension]](https://github.com/Leathong/openscad-support-vscode)

Tested with lsp-mode on Emacs on Linux by [@Lenbok](https://github.com/Lenbok).

Features
--------

-   builtin fucntion/module documents
-   code and path auto-completion
-   jump to definition
-   code snippets
-   function/module signatures on hover
-   document symbols
-   formatter, utilizing clang-format, you need install it youself, it is not built-in.
-   hover and suggestion documentation, read from comments before the function/module.</br>

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
        --builtin <BUILTIN>        external builtin functions file path, if set, the built-in
                                   builtin functions file will not be used [default: ]
        --fmt-exe <FMT_EXE>        clang format executable file path [default: clang-format]
        --fmt-style <FMT_STYLE>    LLVM, GNU, Google, Chromium, Microsoft, Mozilla, WebKit, file
                                   [default: Microsoft]
    -h, --help                     Print help information
        --ip <IP>                  [default: 127.0.0.1]
    -p, --port <PORT>              [default: 3245]
        --stdio                    use stdio instead of tcp
    -V, --version                  Print version information
```

To change the config during running, you can send notification `workspace/didChangeConfiguration` 

```json
{
    "settings": {
        "openscad": {
            "search_paths": "libs",
            "fmt_exe": "fmt_exe",
            "fmt_style": "fmt_style"
        }
    }
}
```

If you work with vscode, you can install the extension directly from the [marketplace](https://marketplace.visualstudio.com/items?itemName=Leathong.openscad-language-support&ssr=false#overview)
