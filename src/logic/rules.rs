//! Fixed rules: from the name of a folder or a file, its family and whether it can be freed.
//! Deliberately short and readable; each rule carries its reason, shown to the user.

use crate::{
    lang::tr,
    logic::tree::{Category, Reclaim},
};

/// Why an item can be freed. Kept as a value, so that it is worded in the interface's language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    NodeModules,
    RustTarget,
    PythonCache,
    Gradle,
    WebBuild,
    CargoRegistry,
    NpmCache,
    XcodeDerived,
    UserCache,
    MacCaches,
    WindowsTemp,
    Trash,
    Models,
    Games,
    Containers,
    DiskImage,
}

impl Why {
    pub fn text(self) -> &'static str {
        match self {
            Why::NodeModules => tr("npm dependencies: npm install puts them back.", "Dépendances npm : se réinstallent avec npm install."),
            Why::RustTarget => tr("Rust build output: cargo build makes it again.", "Compilation Rust : cargo build la refait."),
            Why::PythonCache => tr("Python cache: Python rebuilds it on its own.", "Cache Python : Python le reconstruit tout seul."),
            Why::Gradle => tr("Gradle cache: the next build downloads it again.", "Cache Gradle : le prochain build le retélécharge."),
            Why::WebBuild => tr("Web build cache: the next build makes it again.", "Cache de build web : le prochain build le refait."),
            Why::CargoRegistry => tr("Downloaded crate sources: cargo fetches them again when needed.", "Sources des crates : cargo les retélécharge au besoin."),
            Why::NpmCache => tr("npm download cache: npm fills it again when needed.", "Cache des téléchargements npm : npm le remplit de nouveau au besoin."),
            Why::XcodeDerived => tr("Xcode build output: the next build makes it again.", "Compilation Xcode : le prochain build la refait."),
            Why::UserCache => tr("User cache: applications rebuild it.", "Cache utilisateur : les applications le reconstruisent."),
            Why::MacCaches => tr("macOS caches: applications rebuild them.", "Caches de macOS : les applications les reconstruisent."),
            Why::WindowsTemp => tr("Windows temporary files: nothing needs them once closed.", "Fichiers temporaires de Windows : plus rien ne s'en sert une fois fermés."),
            Why::Trash => tr("Trash: already deleted, it only waits to be emptied.", "Corbeille : déjà supprimé, n'attend plus que d'être vidé."),
            Why::Models => tr("Local AI models: downloadable again, but slow to get back.", "Modèles d'IA locaux : retéléchargeables, mais longs à récupérer."),
            Why::Games => tr("Installed games: remove those you no longer play.", "Jeux installés : désinstallez ceux auxquels vous ne jouez plus."),
            Why::Containers => tr("Container images and volumes: their prune command frees the unused ones.", "Images et volumes de conteneurs : leur commande prune libère ceux qui ne servent plus."),
            Why::DiskImage => tr("Disk image over 500 MB: often useless once installed.", "Image disque de plus de 500 Mo : souvent inutile une fois installée."),
        }
    }
}

/// The rules as the interface lists them: what they look for (`parent/name` when the parent counts,
/// `*.ext` for a file), and why. A test keeps them in step with `classify_dir` and `review_file`.
pub const LISTED: &[(Reclaim, &[&str], Why)] = &[
    (Reclaim::Safe, &["node_modules"], Why::NodeModules),
    (Reclaim::Safe, &["target"], Why::RustTarget),
    (Reclaim::Safe, &["__pycache__", ".pytest_cache", ".mypy_cache", ".ruff_cache"], Why::PythonCache),
    (Reclaim::Safe, &[".gradle"], Why::Gradle),
    (Reclaim::Safe, &[".next", ".turbo", ".svelte-kit", ".parcel-cache"], Why::WebBuild),
    (Reclaim::Safe, &[".cargo/registry"], Why::CargoRegistry),
    (Reclaim::Safe, &[".npm"], Why::NpmCache),
    (Reclaim::Safe, &["Xcode/DerivedData"], Why::XcodeDerived),
    (Reclaim::Safe, &[".cache"], Why::UserCache),
    (Reclaim::Safe, &["Library/Caches"], Why::MacCaches),
    (Reclaim::Safe, &["Local/Temp"], Why::WindowsTemp),
    (Reclaim::Safe, &[".Trash", "Trash", "$Recycle.Bin"], Why::Trash),
    (Reclaim::Review, &["ollama", "lm-studio", "huggingface"], Why::Models),
    (Reclaim::Review, &["Steam"], Why::Games),
    (Reclaim::Review, &["docker", "containers"], Why::Containers),
    (Reclaim::Review, &["*.iso", "*.img", "*.dmg"], Why::DiskImage),
];

pub struct Verdict {
    pub category: Category,
    pub reclaim: Reclaim,
    pub why: Option<Why>,
}

/// Classifies a **folder** by its name and its parent's.
pub fn classify_dir(name: &str, parent: &str) -> Option<Verdict> {
    use Category::*;
    use Reclaim::*;
    let v = |category, reclaim, why| Some(Verdict { category, reclaim, why: Some(why) });
    let kind = |category| Some(Verdict { category, reclaim: No, why: None });
    match (parent, name) {
        (_, "node_modules") => v(Dev, Safe, Why::NodeModules),
        (_, "target") if !parent.is_empty() => v(Dev, Safe, Why::RustTarget),
        (_, "__pycache__" | ".pytest_cache" | ".mypy_cache" | ".ruff_cache") => v(Dev, Safe, Why::PythonCache),
        (_, ".gradle") => v(Dev, Safe, Why::Gradle),
        (_, ".next" | ".turbo" | ".svelte-kit" | ".parcel-cache") => v(Dev, Safe, Why::WebBuild),
        (".cargo", "registry") => v(Dev, Safe, Why::CargoRegistry),
        (_, ".npm") => v(Cache, Safe, Why::NpmCache),
        ("Xcode", "DerivedData") => v(Dev, Safe, Why::XcodeDerived),
        (_, ".cache") => v(Cache, Safe, Why::UserCache),
        ("Library", "Caches") => v(Cache, Safe, Why::MacCaches),
        ("Local", "Temp") => v(Cache, Safe, Why::WindowsTemp),
        (_, ".Trash" | "Trash" | "$Recycle.Bin") => v(Other, Safe, Why::Trash),
        (_, "ollama" | "lm-studio" | "huggingface") => v(Models, Review, Why::Models),
        (_, "Steam") => v(Apps, Review, Why::Games),
        (_, "docker" | "containers") => v(Dev, Review, Why::Containers),
        (_, "Downloads" | "Téléchargements" | "Documents") => kind(Documents),
        (_, "Pictures" | "Images" | "Music" | "Musique" | "Videos" | "Vidéos" | "Movies") => kind(Media),
        (_, "flatpak" | "Applications" | "Program Files" | "Program Files (x86)") => kind(Apps),
        _ => None,
    }
}

/// Classifies a **file** by its extension.
pub fn classify_file(name: &str) -> Category {
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("mp4" | "mkv" | "mov" | "avi" | "webm" | "jpg" | "jpeg" | "png" | "heic" | "raw" | "cr2" | "nef" | "arw" | "mp3" | "flac" | "wav" | "ogg") => Category::Media,
        Some("pdf" | "doc" | "docx" | "odt" | "xlsx" | "ods" | "md" | "txt" | "typ" | "epub") => Category::Documents,
        Some("zip" | "tar" | "gz" | "xz" | "zst" | "7z" | "rar" | "iso" | "img" | "dmg") => Category::Archives,
        Some("gguf" | "safetensors" | "onnx") => Category::Models,
        _ => Category::Other,
    }
}

/// A large disk image (ISO, IMG, DMG) is often forgotten after an install.
pub fn review_file(name: &str, size: u64) -> Option<Why> {
    let lower = name.to_ascii_lowercase();
    let image = [".iso", ".img", ".dmg"].iter().any(|e| lower.ends_with(e));
    (image && size > 500_000_000).then_some(Why::DiskImage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_listed_rules_are_the_rules_applied() {
        for &(reclaim, names, why) in LISTED {
            for name in names {
                let found = match name.strip_prefix('*') {
                    Some(ext) => review_file(&format!("image{ext}"), 600_000_000).map(|w| (Reclaim::Review, w)),
                    None => {
                        let (parent, name) = name.split_once('/').unwrap_or(("parent", name));
                        classify_dir(name, parent).and_then(|v| v.why.map(|w| (v.reclaim, w)))
                    }
                };
                assert_eq!(found, Some((reclaim, why)), "{name}");
            }
        }
        let whys: std::collections::HashSet<_> = LISTED.iter().map(|r| r.2 as u8).collect();
        assert_eq!(whys.len(), Why::DiskImage as usize + 1, "every reason is listed once");
    }

    #[test]
    fn knows_what_can_go() {
        assert_eq!(classify_dir("node_modules", "zoned").unwrap().reclaim, Reclaim::Safe);
        assert_eq!(classify_dir("target", "bref").unwrap().category, Category::Dev);
        // A folder named `target` at the root of the scan is the user's own, not a build.
        assert!(classify_dir("target", "").is_none());
        assert_eq!(classify_dir("registry", ".cargo").unwrap().why, Some(Why::CargoRegistry));
        // `registry` elsewhere is just a folder.
        assert!(classify_dir("registry", "docker-data").is_none());
        assert_eq!(classify_dir("ollama", "share").unwrap().reclaim, Reclaim::Review);
        assert!(classify_dir("src", "bref").is_none());
        // A known folder that is to keep: a family, no reason.
        let downloads = classify_dir("Téléchargements", "moi").unwrap();
        assert_eq!((downloads.category, downloads.reclaim, downloads.why), (Category::Documents, Reclaim::No, None));
    }

    #[test]
    fn knows_files_by_their_extension() {
        assert_eq!(classify_file("film.MKV"), Category::Media);
        assert_eq!(classify_file("sans-extension"), Category::Other);
        assert_eq!(classify_file(".bashrc"), Category::Other);
        assert_eq!(classify_file("archive.tar.zst"), Category::Archives);
        assert_eq!(classify_file("fin."), Category::Other);
        assert_eq!(review_file("ubuntu-26.04.ISO", 6_000_000_000), Some(Why::DiskImage));
        assert_eq!(review_file("petit.iso", 500_000_000), None, "the limit itself is not large");
        assert_eq!(review_file("notes.txt", 6_000_000_000), None);
    }

    #[test]
    fn every_reason_is_worded_in_both_languages() {
        let all = [Why::NodeModules, Why::RustTarget, Why::PythonCache, Why::Gradle, Why::WebBuild, Why::CargoRegistry, Why::NpmCache, Why::XcodeDerived, Why::UserCache, Why::MacCaches, Why::WindowsTemp, Why::Trash, Why::Models, Why::Games, Why::Containers, Why::DiskImage];
        for why in all {
            let (fr, en) = (why.text(), crate::lang::in_english(|| why.text()));
            assert!(!fr.is_empty() && !en.is_empty() && fr != en, "{why:?}");
            assert!(fr.ends_with('.') && en.ends_with('.'), "{why:?}");
        }
    }
}
