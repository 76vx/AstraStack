use std::{fs::File, io::{self, BufReader, BufWriter}, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use astra_stack::{process_stream, TransformProfile};

#[derive(Parser, Debug)]
#[command(name = "astra-stack-cli", author = "Astra", version, about = "Pipeline de transformacion de datos")] 
struct Args {
    /// Ruta del archivo de entrada; si se omite se usa stdin
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Ruta del archivo de salida; si se omite se usa stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Recortar espacios en blanco
    #[arg(long, default_value_t = true)]
    trim: bool,

    /// Convertir a mayúsculas ASCII
    #[arg(long, default_value_t = false)]
    upper: bool,

    /// Omitir líneas vacías
    #[arg(long, default_value_t = true)]
    drop_empty: bool,

    /// Deduplicar líneas
    #[arg(long, default_value_t = false)]
    dedup: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let profile = TransformProfile {
        trim: args.trim,
        to_upper: args.upper,
        drop_empty: args.drop_empty,
        deduplicate: args.dedup,
    };

    let reader: Box<dyn io::BufRead> = match args.input {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(BufReader::new(io::stdin())),
    };

    let writer: Box<dyn io::Write> = match args.output {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(BufWriter::new(io::stdout())),
    };

    let stats = process_stream(reader, writer, profile)?;
    eprintln!(
        "Listas procesadas: {} | escritas: {} | omitidas: {}",
        stats.read, stats.written, stats.skipped
    );

    Ok(())
}
