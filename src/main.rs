use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::exit;
use flate2::write::GzEncoder;
use flate2::Compression;
use base64::encode;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;

fn main() -> io::Result<()> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <remote_server>", args[0]);
        exit(1);
    }
    let file_path = &args[1];
    let mut remote_server = args[2].clone();

    // Ensure the URL has a scheme
    if !remote_server.starts_with("http://") && !remote_server.starts_with("https://") {
        remote_server = format!("http://{}", remote_server);
    }

    // Extract the filename from the file path
    let file_name = Path::new(file_path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    // Read the file bytes
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Encode the file bytes
    let encoded = encode(&buffer);

    // Compress the encoded data
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(encoded.as_bytes())?;
    let compressed_data = encoder.finish()?;

    // Create headers and add the Filename header
    let mut headers = HeaderMap::new();
    headers.insert("Filename", file_name.parse().unwrap());

    // Send the compressed file to the remote server with the Filename header
    let client = Client::new();
    let response = client.post(&remote_server)
        .headers(headers)
        .body(compressed_data)
        .send()
        .expect("Failed to send data to the server");

    println!("Server response: {:?}", response);
    Ok(())
}
