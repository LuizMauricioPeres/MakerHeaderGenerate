use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use maker_header_gen::{analyse_file, render_ctags, render_hpts, render_stdout, write_mkh};

/// Gera arquivos .mkh de manifesto de símbolos para fontes Harbour (.prg)
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Arquivo .prg ou diretório a processar (recursivo)
    input: PathBuf,

    /// Exibir símbolos e usos no stdout também
    #[arg(short, long)]
    verbose: bool,

    /// Gera arquivo tags (Universal ctags) no diretório de entrada em vez dos .mkh
    #[arg(long)]
    tags: bool,

    /// Gera .vscode/hpts.json com índice de símbolos para extensões VSCode
    #[arg(long)]
    hpts: bool,
}

fn main() {
    let args = Args::parse();

    if args.tags {
        process_tags(&args.input);
        return;
    }

    if args.hpts {
        process_hpts(&args.input);
        return;
    }

    if args.input.is_dir() {
        for entry in prg_entries(&args.input) {
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
                println!("{}", render_stdout(&manifest));
            }
            match write_mkh(path, &manifest) {
                Ok(out) => println!("[ok] {}", out.display()),
                Err(e) => eprintln!("[err] {}: {}", path.display(), e),
            }
        }
        Err(e) => eprintln!("[err] {}: {}", path.display(), e),
    }
}

fn process_tags(input: &Path) {
    let paths: Vec<PathBuf> = if input.is_dir() {
        prg_entries(input).map(|e| e.path().to_path_buf()).collect()
    } else {
        vec![input.to_path_buf()]
    };

    let mut manifests = Vec::new();
    for path in &paths {
        match analyse_file(path) {
            Ok(m) => manifests.push(m),
            Err(e) => eprintln!("[err] {}: {}", path.display(), e),
        }
    }

    let refs: Vec<&_> = manifests.iter().collect();
    let content = render_ctags(&refs);

    let tags_dir = if input.is_dir() {
        input.to_path_buf()
    } else {
        input.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    let tags_path = tags_dir.join("tags");
    match fs::write(&tags_path, &content) {
        Ok(()) => println!("[ok] {} ({} símbolos)", tags_path.display(), manifests.iter().map(|m| m.symbols.len()).sum::<usize>()),
        Err(e) => eprintln!("[err] tags: {}", e),
    }
}

fn process_hpts(input: &Path) {
    let paths: Vec<PathBuf> = if input.is_dir() {
        prg_entries(input).map(|e| e.path().to_path_buf()).collect()
    } else {
        vec![input.to_path_buf()]
    };

    let mut manifests = Vec::new();
    for path in &paths {
        match analyse_file(path) {
            Ok(m) => manifests.push(m),
            Err(e) => eprintln!("[err] {}: {}", path.display(), e),
        }
    }

    let refs: Vec<&_> = manifests.iter().collect();
    let content = render_hpts(&refs);

    let root = if input.is_dir() {
        input.to_path_buf()
    } else {
        input.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    let vscode_dir = root.join(".vscode");
    if let Err(e) = fs::create_dir_all(&vscode_dir) {
        eprintln!("[err] .vscode/: {}", e);
        return;
    }

    let out_path = vscode_dir.join("hpts.json");
    match fs::write(&out_path, &content) {
        Ok(()) => println!(
            "[ok] {} ({} símbolos)",
            out_path.display(),
            manifests.iter().map(|m| m.symbols.len()).sum::<usize>()
        ),
        Err(e) => eprintln!("[err] hpts.json: {}", e),
    }
}

fn prg_entries(dir: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(dir)
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
}
