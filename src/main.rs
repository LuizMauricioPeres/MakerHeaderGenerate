use clap::Parser;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod analyser;
mod emitter;
mod types;

use analyser::analyse_file;

/// Gera arquivos .mkh de manifesto de símbolos para fontes Harbour (.prg)
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Arquivo .prg ou diretório a processar (recursivo)
    input: PathBuf,

    /// Exibir símbolos e usos no stdout também
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    if args.input.is_dir() {
        for entry in WalkDir::new(&args.input)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x.eq_ignore_ascii_case("prg"))
                    .unwrap_or(false)
            })
        {
            process(entry.path(), args.verbose);
        }
    } else {
        process(&args.input, args.verbose);
    }
}

fn process(path: &Path, verbose: bool) {
    match analyse_file(path) {
        Ok(manifest) => {
            if verbose {
                println!("{}", emitter::render_stdout(&manifest));
            }
            match emitter::write_mkh(path, &manifest) {
                Ok(out) => println!("[ok] {}", out.display()),
                Err(e) => eprintln!("[err] {}: {}", path.display(), e),
            }
        }
        Err(e) => eprintln!("[err] {}: {}", path.display(), e),
    }
}
