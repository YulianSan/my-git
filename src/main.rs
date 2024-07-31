use core::panic;
use flate2::read::ZlibDecoder;
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
use std::str;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "add" && !args[2].is_empty() {
        let content = match fs::read_to_string(&args[2]) {
            Ok(content) => content,
            Err(_) => panic!("No can read file"),
        };

        let blob_object = format!("blob {}\0{}", content.len(), content);

        let mut hasher = Sha1::new();
        hasher.update(blob_object.clone().into_bytes());
        let blob_object_hash: String = format!("{:x}", hasher.finalize());
        let mut content_object = ZlibEncoder::new(vec![], Compression::default());
        content_object
            .write_all(blob_object.into_bytes().as_ref())
            .unwrap();
        let content_object = content_object.finish().unwrap();
        let content_object = str::from_utf8(&content_object);

        let directory = &blob_object_hash[..2];
        let filename = &blob_object_hash[2..];
        fs::write(
            format!(".git/objects/{}/{}", directory, filename),
            content_object.unwrap(),
        )
        .unwrap();
    } else if args[1] == "cat-file" {
        if args[2] == "-p" && !args.get(3).expect("Need hash").is_empty() {
            let directory = &args[3][..2];
            let filename = &args[3][2..];

            let file = fs::File::open(
                format!(".git/objects/{}/{}", directory, filename),
            ).expect("Hash invalid");

            let mut content_object = ZlibDecoder::new(file);

            let mut content = String::new();
            content_object.read_to_string(&mut content).expect("Invalid content file");

            let content = content.split_once("\0").expect("Invalid content").1;

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            stdout.write_all(content.as_bytes()).unwrap();
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
