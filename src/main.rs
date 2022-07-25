#![allow(clippy::option_map_unit_fn)]

#[macro_use]
mod server;

use clap::Parser;
use lsp_server::Connection;
use server::*;
use std::error::Error;

#[derive(Parser)]
#[clap(name = "OpenSCAD-LSP")]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(short, long, default_value_t = String::from("3245"))]
    port: String,

    #[clap(long, default_value_t = String::from("127.0.0.1"))]
    ip: String,

    #[clap(long, default_value_t = String::from("Microsoft"), help = "LLVM, GNU, Google, Chromium, Microsoft, Mozilla, WebKit, file")]
    fmt_style: String,

    #[clap(long, default_value_t = String::from("clang-format"), help = "clang format executable file path")]
    fmt_exe: String,

    #[clap(long, default_value_t = String::from(""), help = "external builtin functions file path, if set, the built-in builtin functions file will not be used")]
    builtin: String,

    #[clap(long, help = "use stdio instead of tcp")]
    stdio: bool,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let args = Cli::parse();

    let (connection, io_threads) = if args.stdio {
        Connection::stdio()
    } else {
        log_to_console!("start with socket");
        match Connection::listen(format!("{}:{}", args.ip, args.port)) {
            Ok(res) => res,
            Err(err) => {
                err_to_console!("{}", err);
                return Ok(()); // return an error from main will print it to stderr
            }
        }
    };

    log_to_console!("start sucess");
    let mut server = Server::new(connection, args);
    server.main_loop()?;
    io_threads.join()?;

    err_to_console!("exit");
    Ok(())
}
