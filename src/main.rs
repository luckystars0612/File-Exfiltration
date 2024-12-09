use std::env;
use std::fs::File;
use std::io::{self, Read, Write}; 
use std::path::Path;
use flate2::write::GzEncoder;
use flate2::Compression;
use base64::encode;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use std::process::exit;

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <remote_server>", args[0]);
        exit(1);
    }

    let file_path = &args[1];
    let mut remote_server = args[2].clone();

    if !remote_server.starts_with("http://") && !remote_server.starts_with("https://") {
        remote_server = format!("http://{}", remote_server);
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

        // Prepare headers
        let mut headers = HeaderMap::new();
        headers.insert("Filename", HeaderValue::from_str(&file_name).unwrap());
        headers.insert("Chunk-Index", HeaderValue::from_str(&chunk_index.to_string()).unwrap());

        // Send the chunk
        client.post(&remote_server)
            .headers(headers)
            .body(compressed_data)
            .send()
            .expect("Failed to send chunk");

        println!("Sent chunk {} with {} bytes", chunk_index, bytes_read);
        chunk_index += 1;
    }

    println!("File sent successfully in chunks.");
    Ok(())
}
