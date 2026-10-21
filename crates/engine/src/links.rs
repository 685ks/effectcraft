//! Project links (shown in Help ▸, the About dialog and the start screen).

pub const APP_NAME: &str = "EffectCraft";
pub const APP_SLUG: &str = "effectcraft";
/// This fork's repository: the home page, the docs and the issue tracker live here.
pub const GITHUB: &str = "https://github.com/685ks/effectcraft";
pub const APP_PAGE: &str = GITHUB;
pub const ISSUES: &str = "https://github.com/685ks/effectcraft/issues";
/// The project this one is forked from.
pub const UPSTREAM: &str = "https://github.com/storytold/effectcraft";
/// Shown in the About dialog.
pub const FORK_NOTE: &str = "A fork of storytold/effectcraft, not affiliated with or endorsed by ArtCraft.";
/// Shown in the About dialog when a launcher keeps the app up to date.
pub const LAUNCHER_UPDATES_NOTE: &str = "Updates are managed by Arthavan Projects";

/// Sibling Crafting apps: (name, repository slug).
pub const SIBLINGS: &[(&str, &str)] = &[
    ("PhotoCraft", "photocraft"),
    ("VectorCraft", "vectorcraft"),
    ("FilmCraft", "filmcraft"),
    ("LightCraft", "lightcraft"),
    ("PdfCraft", "pdfcraft"),
    ("DesignCraft", "designcraft"),
];

pub fn github_of(slug: &str) -> String {
    format!("https://github.com/685ks/{slug}")
}
pub fn page_of(slug: &str) -> String {
    github_of(slug)
}

/// True when the Arthavan launcher started the app (`ARTHAVAN_LAUNCHER=1`): it owns updates, so
/// the app runs no updater of its own and says so instead.
pub fn launcher_managed() -> bool {
    launcher_managed_from(std::env::var("ARTHAVAN_LAUNCHER").ok().as_deref())
}

pub fn launcher_managed_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.trim() == "1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_point_at_the_fork_and_never_at_artcraft_services() {
        let mut all = vec![GITHUB.to_string(), APP_PAGE.to_string(), ISSUES.to_string()];
        all.extend(SIBLINGS.iter().flat_map(|(_, s)| [github_of(s), page_of(s)]));
        for url in &all {
            assert!(url.starts_with("https://github.com/685ks/"), "{url}");
            assert!(!url.contains("getartcraft") && !url.contains("discord"), "{url}");
        }
        assert!(FORK_NOTE.contains("not affiliated with or endorsed by ArtCraft"));
    }

    #[test]
    fn the_launcher_flag_is_read_from_arthavan_launcher() {
        assert!(launcher_managed_from(Some("1")));
        assert!(launcher_managed_from(Some(" 1\n")));
        for v in [None, Some(""), Some("0"), Some("true"), Some("yes")] {
            assert!(!launcher_managed_from(v), "{v:?}");
        }
    }
}
