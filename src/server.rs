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
    pub pw: String,
}

impl Credential {
    pub fn open(&self) -> Result<Sftp, Box<dyn std::error::Error>> {
        let tcp = TcpStream::connect(&self.host)?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake().unwrap();
        session.userauth_password(&self.username, &self.pw)?;

        let sftp = session.sftp()?;
        Ok(sftp)
    }
}

pub fn latest_dir(ftp: &Sftp) -> Option<PathBuf> {
    let dirs = ftp.readdir(".").ok()?;
    dirs.iter()
        .filter(|x| x.1.is_dir())
        .max_by_key(|(_, stat)| stat.mtime)
        .map(|x| x.0.clone())
}

pub fn save_doc<T: BwarmEntry>(
    ftp: &Sftp,
    in_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let latest_file = in_dir.join(T::filename());
    let file = ftp.open(latest_file)?;
    let outpath = PathBuf::from(T::filename());
    let output = File::open(outpath)?;

    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(output);
    io::copy(&mut reader, &mut writer)?;
    Ok(())
}
