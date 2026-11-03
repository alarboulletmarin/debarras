//! Débarras: see what fills a disk, and free what can go. ARCHITECTURE.md maps the code.

// No console behind the window on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod actions;
mod lang;
mod logic;
mod platform;
mod shell;
mod ui;
mod views;

use std::path::PathBuf;

use lang::tr;

/// What the command line asks for.
#[derive(Debug, PartialEq)]
pub enum Start {
    /// The start screen: choose what to scan.
    Choose,
    Folder(PathBuf),
    Demo,
}

/// Reads the arguments; `Err` holds what to print before leaving (help, version, a mistake).
fn parse(args: &[String]) -> Result<Start, (String, i32)> {
    match args {
        [] => Ok(Start::Choose),
        [flag] if flag == "--demo" => Ok(Start::Demo),
        [flag] if flag == "--version" || flag == "-V" => Err((format!("debarras {}", env!("CARGO_PKG_VERSION")), 0)),
        [flag] if flag == "--help" || flag == "-h" => Err((usage(), 0)),
        [flag, ..] if flag.starts_with('-') && flag != "-" => Err((format!("{} {flag}\n\n{}", tr("Unknown option:", "Option inconnue :"), usage()), 2)),
        [folder] => Ok(Start::Folder(PathBuf::from(folder))),
        _ => Err((usage(), 2)),
    }
}

fn usage() -> String {
    format!(
        "{}\n\n  debarras            {}\n  debarras <{}>   {}\n  debarras --demo     {}\n  debarras --version",
        tr("Débarras: see what fills your disk, and free what can go.", "Débarras : voyez ce qui remplit votre disque, et libérez ce qui peut partir."),
        tr("choose a folder to scan", "choisir un dossier à analyser"),
        tr("folder", "dossier"),
        tr("scan this folder", "analyser ce dossier"),
        tr("a made-up disk, to look around", "un disque fictif, pour découvrir"),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse(&args) {
        Ok(start) => shell::run(start),
        Err((text, code)) => {
            if code == 0 { println!("{text}") } else { eprintln!("{text}") }
            std::process::exit(code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn reads_the_command_line() {
        assert_eq!(parse(&args(&[])), Ok(Start::Choose));
        assert_eq!(parse(&args(&["--demo"])), Ok(Start::Demo));
        assert_eq!(parse(&args(&["/home/moi/projets été"])), Ok(Start::Folder("/home/moi/projets été".into())));
        // A folder whose name starts with a dash is reached as ./-name; `-` alone is a name.
        assert_eq!(parse(&args(&["-"])), Ok(Start::Folder("-".into())));
        assert_eq!(parse(&args(&["--version"])).unwrap_err().1, 0);
        assert!(parse(&args(&["--help"])).unwrap_err().0.contains("--demo"));
        assert_eq!(parse(&args(&["--dark"])).unwrap_err().1, 2);
        assert_eq!(parse(&args(&["a", "b"])).unwrap_err().1, 2);
    }
}
