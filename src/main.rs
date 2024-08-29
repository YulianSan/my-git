use anyhow::Context;
use clap::{Parser, Subcommand};
use core::str;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fs;
use std::io::{prelude::*, BufReader};

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
        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file_name: String,
    },
}

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            init()?;
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(pretty_print, "invalid argumenst`s or options");

            let content = cat_file(object_hash)?;
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            stdout.write_all(&content).unwrap();

        }
        Command::HashObject { write, file_name } => {
            anyhow::ensure!(write, "invalid argumenst`s or options");

            hash_object(file_name)?;
        }
    };

    Ok(())
}

fn init() -> anyhow::Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized git directory");

    Ok(())
}

fn cat_file(object_hash: String) -> anyhow::Result<Vec<u8>> {
    let file = fs::File::open(format!(".git/objects/{}/{}", &object_hash[..2], &object_hash[2..]))
        .context("file open in .git/objects/")?;

    let file_decoder = ZlibDecoder::new(file);

    let mut buffer_decoder = BufReader::new(file_decoder);
    let mut header: Vec<u8> = vec![];
    buffer_decoder.read_until(0, &mut header)?;

    // catch up \0
    let header = CStr::from_bytes_with_nul(&header)
        .expect("know there is exactly one nul, and it's at the end");

    let header = header.to_str().expect("Invalid content Header");
    let header = header.split(" ").collect::<Vec<&str>>();

    let (_content_type, size) = match &header[..] {
        ["blob", size] => (
            Kind::Blob,
            size.parse::<usize>().expect("Excepted size as integer"),
        ),
        [content_type, _] => anyhow::bail!("Invalid file type header: {}", content_type),
        _ => anyhow::bail!("Invalid format header, expected {{content_type}} {{size}}\\0"),
    };

    let mut content = vec![0u8; size];

    buffer_decoder
        .read_exact(&mut content)
        .expect(&format!("Invalid size of content, expected {}", size));

    Ok(content)
}

fn hash_object(file_name: String) -> anyhow::Result<()> {
    let mut file = fs::File::open(&file_name).context(format!("file {} not found", &file_name))?;

    let mut content = vec![];
    file.read_to_end(&mut content)?;
    let content = format!(
        "blob {}\0{}",
        content.len(),
        str::from_utf8(&content).expect("Content invalid utf8")
    );
    let mut hash = Sha1::new();
    hash.write(content.as_bytes())?;
    let hash_name = format!("{:02x}", hash.clone().finalize());
    let mut file_encoder = ZlibEncoder::new(Vec::new(), Compression::default());

    file_encoder.write(content.as_bytes()).unwrap();

    let buffer_encode = file_encoder.finish()?;

    let path_object = format!(".git/objects/{}/", &hash_name[..2]);

    fs::create_dir_all(&path_object).unwrap();
    fs::write(format!("{}{}", path_object, &hash_name[2..]), buffer_encode).unwrap();

    Ok(())
}

#[cfg(test)]
mod test {
    use core::str;
    use std::{env, path};

    use super::*;

    #[test]
    fn should_init() {
        if !path::Path::new(".test").exists() {
            fs::create_dir(".test").unwrap();
        }

        env::set_current_dir(".test").expect("File .test not found");

        init().ok();

        assert_eq!(
            path::Path::new(".git").exists(),
            true,
            "Expected folder .git be created"
        );
        assert_eq!(
            path::Path::new(".git/objects").exists(),
            true,
            "Expected folder .git/objects be created"
        );
        assert_eq!(
            path::Path::new(".git/refs").exists(),
            true,
            "Expected folder .git/refs be created"
        );
        assert_eq!(
            path::Path::new(".git/HEAD").exists(),
            true,
            "Expected file .git/HEAD be created"
        );
        assert_eq!(
            str::from_utf8(&fs::read(".git/HEAD").unwrap()).unwrap(),
            "ref: refs/heads/main\n",
            "Expected file .git/HEAD has content"
        );

        fs::write("test.txt", "hello world").unwrap();

        hash_object("test.txt".to_string()).map_err(|err| {
            eprintln!("{:?}", err);
        }).unwrap();

        let mut files = 0;
        let mut object_hash = String::new();
        for file in fs::read_dir(".git/objects").unwrap() {
            let file = file.unwrap();
            let path = file.path();

            if path.is_dir() {
                object_hash.push_str(path.file_name().unwrap().to_str().unwrap());
                for file in fs::read_dir(path).unwrap() {
                    files += 1;
                    let file = file.unwrap();
                    let path = file.path();
                    object_hash.push_str(path.file_name().unwrap().to_str().unwrap());
                }
            }
        }
        assert_eq!(1, files);

        let content = cat_file(object_hash).unwrap();

        assert_eq!(content, "hello world".as_bytes());
        env::set_current_dir("..").expect("Try out folder .test");
        fs::remove_dir_all(".test").unwrap();
    }
}
