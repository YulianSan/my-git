use std::{fs::File, io::BufReader};
use flate2::read::ZlibDecoder;

pub trait Object {
    fn serialize(&self, buf: BufReader<File>) -> anyhow::Result<String>;
    fn deserialize(&self, hash: &str) -> anyhow::Result<(BufReader<ZlibDecoder<File>>, usize)>;
}
