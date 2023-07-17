use std::path::{Path, PathBuf};

use anyhow::bail;
use clap::Parser;
use rand::RngCore;
use tokio::fs::{create_dir_all, read_dir, remove_file};
static KILO: u64 = 1024;
static MEGA: u64 = 1024 * 1024;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut root = PathBuf::from(std::env::current_dir().unwrap());
    root.push(&args.dir);
    println!("{args:?}");
    println!("{root:?}");

    create_dir_all(&root).await?;
    let root = root.canonicalize()?;

    let mut contents = read_dir(&root).await?;

    if let Some(next) = contents.next_entry().await? {
        if args.force {
            remove_file(next.path()).await?;
            while let Some(next) = contents.next_entry().await? {
                remove_file(next.path()).await?;
            }
        } else {
            bail!("destination is not empty! use --force to remove the contents");
        }
    }
    for i in 0..args.count {
        let filename = format!("gen_{}_{}_{:?}.auto_gen", i, args.size, args.unit);
        let mut complete_path = root.clone();
        complete_path.push(filename);

        make_file(complete_path, args.size, args.unit).await?;
    }
    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    dir: String,
    #[arg(short, long)]
    count: u64,
    #[arg(short, long, value_enum, default_value_t)]
    unit: UnitType,
    #[arg(short, long)]
    size: u64,
    #[clap(long, short, action)]
    force: bool,
}

#[derive(Debug, Default, Clone, Copy, Parser, clap::ValueEnum)]
enum UnitType {
    Bytes,
    Kilo,
    #[default]
    Mega,
}

async fn make_file(path: impl AsRef<Path>, size: u64, unit: UnitType) -> std::io::Result<()> {
    let size = real_size(size, unit);

    let mut contents = Vec::<u8>::with_capacity(size as usize);

    for _ in 0..size {
        contents.push(0);
    }

    rand::thread_rng().fill_bytes(&mut contents);

    tokio::fs::write(path, &contents).await?;
    Ok(())
}

fn real_size(size: u64, unit: UnitType) -> u64 {
    size * match unit {
        UnitType::Bytes => 1,
        UnitType::Kilo => KILO,
        UnitType::Mega => MEGA,
    }
}
