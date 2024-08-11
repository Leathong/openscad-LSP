#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::collapsible_if)]

#[macro_use]
mod server;

use clap::Parser;
use lsp_server::Connection;
use server::*;
use std::error::Error;

#[derive(Parser)]
#[command(name = "OpenSCAD-LSP")]
#[command(author, version, about)]
pub(crate) struct Cli {
    #[arg(short, long, default_value_t = String::from("3245"))]
    port: String,

    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    ip: String,

    #[arg(
        long,
        value_parser = ["LLVM", "GNU", "Google", "Chromium", "Microsoft", "Mozilla", "Webkit", "file"],
        help = "formatting style for clang-format",
    )]
    fmt_style: Option<String>,

    #[arg(
        long,
        default_value = "clang-format",
        help = "formatter executable file path"
    )]
    fmt_exe: String,

    #[arg(long, help = "formatter executable arguments")]
    fmt_args: Vec<String>,

    #[arg(
        long,
        help = "external builtin functions file path, if set, the built-in builtin functions file will not be used"
    )]
    builtin: Option<String>,

    #[arg(long, help = "use stdio instead of tcp")]
    stdio: bool,

    #[arg(long, help = "exclude default params in auto-completion")]
    ignore_default: bool,

    #[arg(long, default_value_t = 3, help = "search depth")]
    depth: i32,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let args = Cli::parse();

    let (connection, io_threads) = if args.stdio {
        Connection::stdio()
    } else {
        log_to_console!("Start with socket");
        match Connection::listen(format!("{}:{}", args.ip, args.port)) {
            Ok(res) => res,
            Err(err) => {
                err_to_console!("{}", err);
                return Ok(()); // return an error from main will print it to stderr
            }
        }
    };

    log_to_console!("Start successful");
    Server::create_server(connection, args);
    Server::get_server().main_loop()?;
    io_threads.join()?;

    err_to_console!("exit");
    Ok(())
}
