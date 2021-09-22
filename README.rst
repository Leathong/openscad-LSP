##########################
 openscad-language-server
##########################

openscad-language-server is an LSP_ (Language Server Protocol) server
for OpenSCAD_. It enables IDE-style features for OpenSCAD code in any
editor for which an LSP client is available—i.e., `most major modern
text editors <clients_>`_.

*Status*: Pretty basic by IDE standards, but functional and well beyond
what the built-in OpenSCAD editor provides. Tested primarily with
lsp-mode_ on Emacs_ on Linux.

**********
 Features
**********

-  context-aware completion
-  insertion of useful snippets upon completion
-  live diagnostic messages for syntax errors
-  function/module signatures on hover
-  handling of ``include``/``use`` (of both local and library files)
-  reasonable robustness in the presence of ill-formed input files

**************
 Installation
**************

openscad-language-server is written in Rust_ and does not depend on
OpenSCAD. Currently, in order to use it, you need to `have a Rust
toolchain installed <install-rust_>`_.

To install the server from crates.io_ (puts the binary into
``~/.cargo/bin/openscad-language-server``):

.. code:: sh

   cargo install openscad-language-server

To build directly from the repository (puts the binary into
``target/release/openscad-language-server`` in the clone):

.. code:: sh

   git clone https://github.com/dzhu/openscad-language-server
   cd openscad-language-server
   cargo build --release

*******
 Usage
*******

Consult the documentation for your editor and its LSP client to
configure them to use the server binary for OpenSCAD files. The server
communicates over standard input/output.

*****************
 Acknowledgments
*****************

Parsing of OpenSCAD code is handled by tree-sitter_ and
tree-sitter-openscad_. Communicating over LSP is handled by lsp-server_.
Having those crates handling all the dirty details of interacting with
the outside world has made it possible to get started on this project
quite quickly and stay focused on the interesting parts in the middle.

**************
 Related work
**************

-  https://github.com/openscad/openscad/pull/3635 (PR for adding LSP
   server functionality into OpenSCAD itself)
-  https://github.com/Antyos/vscode-openscad (plugin for VSCode)
-  https://github.com/ncsaba/idea-openscad (plugin for IntelliJ IDEs)
-  https://github.com/tralamazza/Sublime-OpenScad (syntax for Sublime
   Text)
-  https://github.com/Maxattax97/openscad-lsp (only a skeleton, no
   functionality)

.. _clients: https://langserver.org/#implementations-client

.. _crates.io: https://crates.io

.. _emacs: https://www.gnu.org/software/emacs/

.. _install-rust: https://www.rust-lang.org/learn/get-started

.. _lsp: https://microsoft.github.io/language-server-protocol/

.. _lsp-mode: https://emacs-lsp.github.io/lsp-mode/

.. _lsp-server: https://github.com/rust-analyzer/lsp-server

.. _openscad: https://openscad.org

.. _rust: https://rust-lang.org

.. _tree-sitter: https://tree-sitter.github.io

.. _tree-sitter-openscad: https://github.com/bollian/tree-sitter-openscad
