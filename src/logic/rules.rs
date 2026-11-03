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
            Why::NodeModules => tr("npm dependencies, installed again by npm install.", "Dépendances npm, réinstallées par npm install."),
            Why::RustTarget => tr("Rust build output, rebuilt by cargo build.", "Compilation Rust, refaite par cargo build."),
            Why::PythonCache => tr("Python cache, rebuilt on its own.", "Cache Python, reconstruit tout seul."),
            Why::Gradle => tr("Gradle cache, downloaded again by the next build.", "Cache Gradle, retéléchargé au prochain build."),
            Why::WebBuild => tr("Web build cache, rebuilt by the next build.", "Cache de build web, refait au prochain build."),
            Why::CargoRegistry => tr("Downloaded crate sources, fetched again when needed.", "Sources des crates téléchargées, retéléchargées au besoin."),
            Why::NpmCache => tr("npm download cache, filled again when needed.", "Cache des téléchargements npm, rempli de nouveau au besoin."),
            Why::XcodeDerived => tr("Xcode build output, rebuilt by the next build.", "Compilation Xcode, refaite au prochain build."),
            Why::UserCache => tr("User cache: applications rebuild it.", "Cache utilisateur : les applications le reconstruisent."),
            Why::MacCaches => tr("macOS caches: applications rebuild them.", "Caches de macOS : les applications les reconstruisent."),
            Why::WindowsTemp => tr("Windows temporary files.", "Fichiers temporaires de Windows."),
            Why::Trash => tr("Trash: already deleted, not yet emptied.", "Corbeille : déjà supprimé, pas encore vidé."),
            Why::Models => tr("Local AI models: large and downloadable again, but slow to get back.", "Modèles d'IA locaux : volumineux et retéléchargeables, mais longs à récupérer."),
            Why::Games => tr("Installed games: remove those you no longer play.", "Jeux installés : désinstallez ceux auxquels vous ne jouez plus."),
            Why::Containers => tr("Container images and volumes: their prune command frees the unused ones.", "Images et volumes de conteneurs : leur commande prune libère ceux qui ne servent plus."),
            Why::DiskImage => tr("Disk image: often kept after an install.", "Image disque : souvent gardée après une installation."),
        }
    }
}

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
