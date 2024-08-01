use core::panic;
use std::ffi::CStr;
use anyhow::{anyhow, Context};
use flate2::read::ZlibDecoder;
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::{prelude::*, BufReader};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String
    }
}

enum Kind {
    Blob
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // Uncomment this block to pass the first stage
    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        },
        Command::CatFile { pretty_print, object_hash } => {
            anyhow::ensure!(pretty_print, "invalid argumenst`s or options");

            let file = fs::File::open(
                format!(".files/{}/{}", &object_hash[..2], &object_hash[2..]),
            ).context("file open in .git/objects/")?;

            let file_decoder = ZlibDecoder::new(file);

            let mut buffer_decoder = BufReader::new(file_decoder);
            let mut header:Vec<u8> = vec![];
            buffer_decoder.read_until(0, &mut header);

            // catch up \0
            let header = CStr::from_bytes_with_nul(&header)
                .expect("know there is exactly one nul, and it's at the end");

            let header = header.to_str().expect("Invalid content Header");
            let header = header.split(" ").collect::<Vec<&str>>();
            dbg!(&header);
            let (content_type, size) = match &header[..]{
                ["blob", size] => (Kind::Blob, size.parse::<usize>().expect("Excepted size as integer")),
                [content_type, size] => anyhow::bail!("Invalid file type header: {}", content_type),
                _ => anyhow::bail!("Invalid format header, expected {{content_type}} {{size}}\\0")
            };

            let mut content = vec![0u8; size];

            buffer_decoder.read_exact(&mut content).expect(&format!("Invalid size of content, expected {}", size));

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            stdout.write_all(&content).unwrap();
        }
    };

    Ok(())
}
