use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    net::TcpStream,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use ssh2::{Session, Sftp};

use crate::bwarm::interface::BwarmEntry;

#[derive(Debug, Deserialize, Clone)]
pub struct Credential {
    pub host: String,
    pub username: String,
    pub public_key: PathBuf,
    pub private_key: PathBuf,
}

impl Credential {
    pub fn open(&self) -> Result<Sftp, Box<dyn std::error::Error>> {
        let tcp = TcpStream::connect(&self.host)?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake().unwrap();

        session.userauth_pubkey_file(
            &self.username,
            Some(&self.public_key),
            &self.private_key,
            None,
        )?;

        let sftp = session.sftp()?;
        Ok(sftp)
    }
}

pub fn latest_dir(ftp: &Sftp) -> Option<PathBuf> {
    let dirs = ftp.readdir("./public-database-v2").ok()?;
    dirs.iter()
        .filter(|x| x.1.is_dir())
        .max_by_key(|(_, stat)| stat.mtime)
        .map(|x| x.0.clone())
}

pub fn save_doc<T: BwarmEntry>(
    ftp: &Sftp,
    in_dir: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let latest_file = in_dir.join(T::filename());
    println!("looking for {:?}", latest_file);
    let file = ftp.open(latest_file)?;
    let outpath = out_dir.join(T::filename());
    let output = File::create(outpath)?;

    let capacity = 128 * 1024;
    let mut reader = BufReader::with_capacity(capacity, file);
    let mut writer = BufWriter::with_capacity(capacity, output);
    io::copy(&mut reader, &mut writer)?;
    Ok(())
}
