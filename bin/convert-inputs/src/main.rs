/// Converts ZiskStdin .bin files from bincode v1 to bincode v2 wire format.
///
/// The .bin files contain two length-prefixed frames:
///   Frame 1: RethInputPublic
///   Frame 2: RethInputWitness
///
/// Frame layout: [8-byte LE length][data][padding to 8-byte alignment]
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use guest_reth::{RethInputPublic, RethInputWitness};

#[derive(Parser)]
#[command(about = "Convert ZiskStdin .bin files from bincode v1 to bincode v2 format (in-place)")]
struct Args {
    /// .bin files or directories to convert
    #[arg(required = true)]
    paths: Vec<PathBuf>,
}

fn read_frame(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let mut len_bytes = [0u8; 8];
    cursor.read_exact(&mut len_bytes).context("failed to read frame length")?;
    let len = usize::from_le_bytes(len_bytes);
    let mut data = vec![0u8; len];
    cursor.read_exact(&mut data).context("failed to read frame data")?;
    let padding = (8 - ((8 + len) % 8)) % 8;
    if padding > 0 {
        let mut pad = vec![0u8; padding];
        cursor.read_exact(&mut pad).context("failed to read frame padding")?;
    }
    Ok(data)
}

fn write_frame(out: &mut Vec<u8>, data: &[u8]) {
    let len = data.len();
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(data);
    let padding = (8 - ((8 + len) % 8)) % 8;
    out.extend_from_slice(&vec![0u8; padding]);
}

fn convert_file(path: &Path) -> Result<()> {
    let raw = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut cursor = Cursor::new(raw.as_slice());

    let public_v1 = read_frame(&mut cursor).context("frame 1 (RethInputPublic)")?;
    let public: RethInputPublic = bincode_v1::deserialize(&public_v1)
        .context("failed to deserialize RethInputPublic with bincode v1")?;

    let witness_v1 = read_frame(&mut cursor).context("frame 2 (RethInputWitness)")?;
    let witness: RethInputWitness = bincode_v1::deserialize(&witness_v1)
        .context("failed to deserialize RethInputWitness with bincode v1")?;

    let public_v2 = RethInputPublic::serialize(&public)
        .context("failed to re-serialize RethInputPublic with bincode v2")?;
    let witness_v2 = RethInputWitness::serialize(&witness)
        .context("failed to re-serialize RethInputWitness with bincode v2")?;

    let mut out = Vec::with_capacity(public_v2.len() + witness_v2.len() + 32);
    write_frame(&mut out, &public_v2);
    write_frame(&mut out, &witness_v2);

    fs::write(path, &out).with_context(|| format!("failed to write {}", path.display()))?;
    println!(
        "  {} ({} -> {} bytes)",
        path.file_name().unwrap_or_default().to_string_lossy(),
        raw.len(),
        out.len()
    );
    Ok(())
}

fn collect_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            for entry in fs::read_dir(path)
                .with_context(|| format!("failed to read directory {}", path.display()))?
            {
                let p = entry?.path();
                if p.extension().is_some_and(|e| e == "bin") {
                    files.push(p);
                }
            }
        } else {
            files.push(path.clone());
        }
    }
    files.sort();
    Ok(files)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let files = collect_files(&args.paths)?;

    if files.is_empty() {
        eprintln!("no .bin files found");
        return Ok(());
    }

    println!("Converting {} file(s):", files.len());
    let mut errors = 0usize;
    for file in &files {
        if let Err(e) = convert_file(file) {
            eprintln!("  ERROR {}: {:#}", file.display(), e);
            errors += 1;
        }
    }

    if errors > 0 {
        anyhow::bail!("{} file(s) failed to convert", errors);
    }
    println!("Done.");
    Ok(())
}
