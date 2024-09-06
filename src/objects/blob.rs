use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    ffi::CStr,
    fs::{self, File},
    io::{prelude::*, BufReader},
    str
};

use crate::objects::traits;
use anyhow::Context;

pub struct Blob {}

impl traits::Object for Blob {
    fn serialize(&self, mut file: BufReader<File>) -> anyhow::Result<String> {
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
        Ok(hash_name)
    }

    fn deserialize(
        &self,
        object_hash: &str,
    ) -> anyhow::Result<(BufReader<ZlibDecoder<File>>, usize)> {
        let file = fs::File::open(format!(
            ".git/objects/{}/{}",
            &object_hash[..2],
            &object_hash[2..]
        ))
        .context("file open in .git/objects/")?;

        let file_decoder = ZlibDecoder::new(file);

        let mut buffer_decoder = BufReader::new(file_decoder);
        let mut header: Vec<u8> = vec![];
        buffer_decoder.read_until(0, &mut header)?;

        let header = CStr::from_bytes_with_nul(&header)
            .expect("know there is exactly one nul, and it's at the end");

        let header = header.to_str().expect("Invalid content Header");
        let header = header.split(" ").collect::<Vec<&str>>();

        let size = match &header[..] {
            ["blob", size] => size.parse::<usize>().expect("Excepted size as integer"),
            [content_type, _] => anyhow::bail!("Invalid file type header: {}", content_type),
            _ => anyhow::bail!("Invalid format header, expected {{content_type}} {{size}}\\0"),
        };

        Ok((buffer_decoder, size))
    }
}
