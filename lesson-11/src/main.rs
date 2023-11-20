use server::run_server; 
use client::run_client;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: {} <server/client> <address>", args[0]);
        process::exit(1);
    }

    let mode = &args[1];
    let address = &args[2];

    match mode.as_str() {
        "server" => {
            server::run_server(address);
        }
        "client" => {
            client::run_server(address);
        }
        _ => {
            println!("Invalid mode. Use 'server' or 'client'.");
            process::exit(1);
        }
    }
}
