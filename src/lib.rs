use anyhow::Context;
use clap::{Parser, Subcommand};
use core::str;
use std::io::{prelude::*, BufReader};
use std::{fs, io};

mod objects;

use crate::objects::blob::Blob;

enum Kind {
    Blob,
}

impl Kind {
    fn to_object(kind: Self) -> Box<dyn objects::traits::Object> {
        match kind {
            Kind::Blob => Box::new(Blob {}),
            _ => panic!("Kind invalid, expect Blob, Tree, Commit or Tag"),
        }
    }
}

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

pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            init(None)?;
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(pretty_print, "invalid argumenst`s or options");

            let object = Kind::to_object(Kind::Blob);

            let mut content = object.deserialize(&object_hash)?;
            let mut stdout = std::io::stdout();

            io::copy(&mut content.0, &mut stdout.lock())?;
            stdout.flush()?;
        }
        Command::HashObject { write, file_name } => {
            anyhow::ensure!(write, "invalid argumenst`s or options");

            let object = Kind::to_object(Kind::Blob);
            let file = BufReader::new(
                fs::File::open(&file_name).context(format!("file {} not found", &file_name))?,
            );

            let hash_name = object.serialize(file)?;

            let mut stdout = std::io::stdout().lock();

            stdout.write(hash_name.as_bytes())?;
            stdout.flush()?;
        }
    };

    Ok(())
}

pub fn init(path: Option<&str>) -> anyhow::Result<()> {
    let path = path.unwrap_or(".");

    fs::create_dir(format!("{}/.git", path))?;
    fs::create_dir(format!("{}/.git/objects", path))?;
    fs::create_dir(format!("{}/.git/refs", path))?;
    fs::write(format!("{}/.git/HEAD", path), "ref: refs/heads/main\n")?;

    println!("Initialized git directory");

    Ok(())
}
