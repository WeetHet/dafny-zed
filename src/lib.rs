use std::fs;
use zed_extension_api::{self as zed, Result};

struct DafnyExtension {
    cached_binary_path: Option<String>,
}

#[derive(Clone)]
struct DafnyBinary(String);

impl DafnyExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<DafnyBinary> {
        if let Some(path) = worktree.which("dafny") {
            return Ok(DafnyBinary(path));
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(DafnyBinary(path.clone()));
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "dafny-lang/dafny",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let version = release
            .version
            .strip_prefix("v")
            .unwrap_or(&release.version);
        let asset_name = format!(
            "dafny-{version}-{arch}-{os}.zip",
            version = version,
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X8664 => "x64",
                zed::Architecture::X86 => return Err("unsupported architecture".into()),
            },
            os = match platform {
                zed::Os::Mac => "macos-11",
                zed::Os::Linux => "ubuntu-20.04",
                zed::Os::Windows => "windows-2019",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("dafny-{}", release.version);
        fs::create_dir_all(&version_dir).map_err(|e| format!("failed to create directory: {e}"))?;

        let binary_path = format!(
            "{version_dir}/dafny/{dafny_binary}",
            dafny_binary = match platform {
                zed::Os::Mac => "dafny",
                zed::Os::Linux => "dafny",
                zed::Os::Windows => "Dafny.exe",
            }
        );

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;
            if let Ok(z3s) = fs::read_dir(format!("{version_dir}/dafny/z3/bin")) {
                for file in z3s.flatten() {
                    if let Some(path) = file.path().to_str() {
                        zed::make_file_executable(path)?;
                    }
                }
            }

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(DafnyBinary(binary_path))
    }
}

impl zed::Extension for DafnyExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let DafnyBinary(path) = self.language_server_binary(language_server_id, worktree)?;
        Ok(zed::Command {
            command: path,
            args: vec!["server".to_string()],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(DafnyExtension);
