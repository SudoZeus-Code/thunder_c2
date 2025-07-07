use std::env;

mod client;
use client::client_main;

mod server;
use server::server_main;

fn main() {

    // use collect() to turn the iterator into a vector containing all the values produced by the iterator. 
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("<!> args accepts 'server' or 'client'");
        return;
    }
    // Here we will match either 'server' or 'client' input and direct to new functions accordingly.

    // args[0] in rust is reserved for the program name or path. 
    match args[1].as_str() {
        "server" => server_main(),
        "client" => client_main(),
        _ => println!("<!> args accept 'server' or 'client'"),
    }
    
}
