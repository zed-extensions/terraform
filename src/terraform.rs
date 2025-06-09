use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use zed::ContextServerId;
use zed::LanguageServerId;
use zed_extension_api::{self as zed, serde_json, ContextServerConfiguration, Project, Result};

struct TerraformExtension {
    cached_mcp_binary_path: Option<String>,
    cached_ls_binary_path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TerraformContextServerSettings {}

impl TerraformExtension {
    fn context_server_binary_path(
        &mut self,
        context_server_id: &ContextServerId,
    ) -> Result<String> {
        if let Some(path) = &self.cached_mcp_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let release = zed::latest_github_release(
            "hashicorp/terraform-mcp-server",
            zed::GithubReleaseOptions {
                require_assets: false,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let download_url = format!(
            "https://releases.hashicorp.com/terraform-mcp-server/{version}/terraform-mcp-server_{version}_{os}_{arch}.zip",
            version = release.version.strip_prefix('v').unwrap_or(&release.version),
            os = match platform {
                zed::Os::Mac => "darwin",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X86 => "386",
                zed::Architecture::X8664 => "amd64",
            },
        );

        let version_dir = format!("terraform-mcp-server-{}", release.version);
        let binary_path = format!("{version_dir}/terraform-mcp-server");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::download_file(&download_url, &version_dir, zed::DownloadedFileType::Zip)
                .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_mcp_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }

    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("terraform-ls") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_ls_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "hashicorp/terraform-ls",
            zed::GithubReleaseOptions {
                require_assets: false,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let download_url = format!(
            "https://releases.hashicorp.com/terraform-ls/{version}/terraform-ls_{version}_{os}_{arch}.zip",
            version = release.version.strip_prefix('v').unwrap_or(&release.version),
            os = match platform {
                zed::Os::Mac => "darwin",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X86 => "386",
                zed::Architecture::X8664 => "amd64",
            },
        );

        let version_dir = format!("terraform-ls-{}", release.version);
        let binary_path = format!("{version_dir}/terraform-ls");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(&download_url, &version_dir, zed::DownloadedFileType::Zip)
                .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_ls_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for TerraformExtension {
    fn new() -> Self {
        Self {
            cached_mcp_binary_path: None,
            cached_ls_binary_path: None,
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        _project: &zed::Project,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.context_server_binary_path(context_server_id)?,
            args: vec!["stdio".to_string()],
            env: Default::default(),
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema =
            serde_json::to_string(&schemars::schema_for!(TerraformContextServerSettings))
                .map_err(|e| e.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec!["serve".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(TerraformExtension);
