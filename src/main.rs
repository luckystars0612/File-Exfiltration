use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use flate2::write::GzEncoder;
use flate2::Compression;
use base64::encode;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use sha2::{Digest, Sha256};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use clap::{Arg, Command};

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let matches = Command::new("File Sender")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Sends files in encrypted chunks to a remote server")
        .arg(Arg::new("password")
            .short('p')
            .long("password")
            .value_name("PASSWORD")
            .help("The password to encrypt the file")
            .required(true))
        .arg(Arg::new("filepath")
            .short('f')
            .long("filepath")
            .value_name("FILEPATH")
            .help("The path to the file to be sent")
            .required(true))
        .arg(Arg::new("target")
            .short('t')
            .long("target")
            .value_name("TARGET")
            .help("The target server URL")
            .required(true))
        .get_matches();

    let password = matches.get_one::<String>("password").unwrap();
    let file_path = matches.get_one::<String>("filepath").unwrap();
    let mut target = matches.get_one::<String>("target").unwrap().clone();

    if !target.starts_with("http://") && !target.starts_with("https://") {
        target = format!("http://{}", target);
    }

    let file_name = Path::new(file_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    let mut file = File::open(file_path)?;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let client = Client::new();
    let mut chunk_index = 0;

    // Derive a 32-byte key and 12-byte nonce from the password
    let key = Sha256::digest(password.as_bytes()); // 32 bytes
    let nonce = &key[0..12]; // Use the first 12 bytes as a nonce

    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        // Base64 encode the chunk
        let encoded = encode(&buffer[..bytes_read]);

        // Compress the encoded data
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(encoded.as_bytes()).unwrap();
        let compressed_data = encoder.finish().unwrap();

        // Encrypt the compressed data with ChaCha20
        let mut cipher = ChaCha20::new_from_slices(&key, nonce).unwrap();
        let mut encrypted_data = compressed_data.clone();
        cipher.apply_keystream(&mut encrypted_data);

        // Prepare headers
        let mut headers = HeaderMap::new();
        headers.insert("Filename", HeaderValue::from_str(&file_name).unwrap());
        headers.insert("Chunk-Index", HeaderValue::from_str(&chunk_index.to_string()).unwrap());

        // Send the chunk
        client.post(&target)
            .headers(headers)
            .body(encrypted_data)
            .send()
            .expect("Failed to send chunk");

        println!("Sent chunk {} with {} bytes", chunk_index, bytes_read);
        chunk_index += 1;
    }

    println!("File sent successfully in chunks.");
    Ok(())
}
