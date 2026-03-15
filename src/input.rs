use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use anyhow::{Context, Result};

const FIFO_PATH: &str = "/tmp/cawave.fifo";

pub struct InputReader {
    pub rx: Receiver<Vec<f32>>,
    _cava: Child, // kept alive, killed on drop
}

impl InputReader {
    pub fn spawn(driver_bars: usize) -> Result<Self> {
        ensure_fifo(FIFO_PATH)?;

        let config_path = write_cava_config(driver_bars)?;

        let cava = Command::new("cava")
            .arg("-p")
            .arg(&config_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("failed to spawn cava — is it installed?")?;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            // Blocks until cava opens its end of the FIFO
            let file = File::open(FIFO_PATH).expect("failed to open FIFO");
            let mut reader = BufReader::new(file);
            let mut buf = vec![0u8; driver_bars * 2];

            loop {
                if reader.read_exact(&mut buf).is_err() {
                    break;
                }
                let frame: Vec<f32> = buf
                    .chunks_exact(2)
                    .map(|b| {
                        let raw = u16::from_ne_bytes([b[0], b[1]]);
                        raw as f32 / 65535.0
                    })
                    .collect();
                if tx.send(frame).is_err() {
                    break;
                }
            }
        });

        Ok(Self { rx, _cava: cava })
    }
}

impl Drop for InputReader {
    fn drop(&mut self) {
        let _ = self._cava.kill();
    }
}

fn ensure_fifo(path: &str) -> Result<()> {
    let p = Path::new(path);
    if p.exists() {
        if !fs::metadata(p)?.file_type().is_fifo() {
            fs::remove_file(p)?;
        } else {
            return Ok(());
        }
    }
    Command::new("mkfifo")
        .arg(path)
        .status()
        .context("failed to create FIFO — mkfifo not available?")?;
    Ok(())
}

fn write_cava_config(driver_bars: usize) -> Result<PathBuf> {
    let path = PathBuf::from("/tmp/cawave-cava.ini");
    let config = format!(
        "[general]\nbars = {driver_bars}\n\n[output]\nmethod = raw\nraw_target = {FIFO_PATH}\ndata_format = binary\nbits = 16\n"
    );
    let mut f = File::create(&path)?;
    f.write_all(config.as_bytes())?;
    Ok(path)
}
