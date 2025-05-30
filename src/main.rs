#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::collapsible_if)]

#[macro_use]
mod server;
mod topiary;

use clap::Parser;
use lsp_server::Connection;
use server::*;
use std::{error::Error, path::PathBuf};

#[derive(Parser)]
#[clap(name = "OpenSCAD-LSP")]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(short, long, default_value_t = String::from("3245"))]
    port: String,

    #[clap(long, default_value_t = String::from("127.0.0.1"))]
    ip: String,

    #[clap(long, default_value_t = String::from(""), help = "external builtin functions file path, if not set, the built-in file will be used")]
    builtin: String,

    #[clap(long, help = "use stdio instead of tcp")]
    stdio: bool,

    #[clap(long, help = "exclude default params in auto-completion")]
    ignore_default: bool,

    #[clap(long, default_value_t = 3, help = "search depth")]
    depth: i32,

    #[clap(
        long,
        default_value_t = String::from("  "),
        help = r#"The indentation string used for that particular language. Defaults to "  " if not provided. Any string can be provided, but in most instances will be some whitespace: "  ", "    ", or "\t"."#
    )]
    indent: String,

    #[clap(long, help = "The query file used for topiary formatting")]
    query_file: Option<PathBuf>,
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
    Server::new(connection, args).main_loop()?;
    io_threads.join()?;

    err_to_console!("exit");
    Ok(())
}
