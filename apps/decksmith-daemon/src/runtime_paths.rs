//! Relocatable installed resources, with an explicit source-checkout fallback.
use std::path::{Path, PathBuf};
pub fn root() -> PathBuf {
    if let Some(root) = std::env::var_os("DECKSMITH_RESOURCE_ROOT") {
        return PathBuf::from(root);
    }
    let executable = std::env::current_exe().unwrap_or_default();
    if let Some(root) = installed_root(&executable) {
        return root;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn installed_root(executable: &Path) -> Option<PathBuf> {
    let executable = executable.canonicalize().ok()?;
    let root = executable.parent()?.parent()?;
    (root.join("release.json").is_file() || executable.parent()?.file_name()? == "bin")
        .then(|| root.to_path_buf())
}
pub fn helper(name: &str) -> PathBuf {
    root().join("apps/decksmith-studio").join(name)
}
pub fn config(name: &str) -> PathBuf {
    root().join("config").join(name)
}
pub fn check() -> Result<serde_json::Value, &'static str> {
    let root = root();
    if !root.is_absolute() {
        return Err("resource_root_must_be_absolute");
    }
    for path in [
        config("audio.json"),
        config("navigation.json"),
        helper("panel.py"),
        helper("audio_targets.py"),
        helper("audio_meter.py"),
        helper("ptt_guard.py"),
        helper("launch.py"),
        helper("system_controls.py"),
        helper("system_confirm.py"),
        helper("control_health.py"),
        root.join("assets/icons/tabler/catalog.json"),
        root.join("brand/decksmith-makers-mark-brand-sheet.png"),
    ] {
        if !path.is_file() {
            return Err("runtime_resource_missing");
        }
    }
    Ok(
        serde_json::json!({"resource_root":root,"resources":"ok","version":env!("CARGO_PKG_VERSION")}),
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn installed_root_is_derived_from_executable_not_working_directory() {
        let dir = std::env::temp_dir().join(format!("decksmith-path-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::write(dir.join("release.json"), b"{}").unwrap();
        std::fs::write(dir.join("bin/decksmithd"), b"fixture").unwrap();
        assert_eq!(
            super::installed_root(&dir.join("bin/decksmithd")),
            Some(dir.canonicalize().unwrap())
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
