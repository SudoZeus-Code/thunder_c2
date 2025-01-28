// Combineing client and sever
// to take user input ( server , client )
// These cli args will be interperted in main()
// calls will then push them to server or client respectively
// once we have some basic functionality for the cli, ...
// we will move the two programs into one. 

// module containing functions to instpect various aspects of environment including processing args. 
use std::env;

fn main() {

    //println!("Hello, world!");

    // use collect() to turn the iterator into a vector containing all the values produced by the iterator. 
    let args: Vec<String> = env::args().collect();

    // print the vector using the debug macro
    //dbg!(args);

    if args.len() < 2 {
        println!("<!> args accepts 'server' or 'client'");
        return;
    }

    // Here we will match either 'server' or 'client' input and direct to new functions accordingly.

    // args[0] in rust is reserved for the program name or path. 
    match args[1].as_str() {
        "server" => server(),
        "client" => client(),
        _ => println!("<!> args accept 'server' or 'client'"),
    }
    

}

fn server() {
    println!("in server block");
}

fn client() {
    println!("in client block");
}






