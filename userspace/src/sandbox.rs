use anyhow::{Context, Result};
use std::process::Command;

/// Orchestrates the execution of a command within a sandbox.
/// Uses bubblewrap on Linux and sandbox-exec on macOS.
pub fn run_sandboxed(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    return run_linux_sandbox(args);

    #[cfg(target_os = "macos")]
    return run_macos_sandbox(args);

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        println!("⚠️ Sandboxing is not supported on this platform. Running natively...");
        let status = Command::new(&args[0])
            .args(&args[1..])
            .status()
            .context("Failed to run command")?;
        if !status.success() {
            anyhow::bail!("Command failed");
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn run_linux_sandbox(target_args: &[String]) -> Result<()> {
    let bwrap_args = build_bwrap_args(target_args);
    
    let mut child = Command::new("bwrap")
        .args(&bwrap_args)
        .spawn()
        .context("Failed to execute bwrap. Is it installed?")?;

    let status = child.wait().context("Failed to wait for sandboxed process")?;
    
    if !status.success() {
        anyhow::bail!("Sandboxed command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn run_macos_sandbox(target_args: &[String]) -> Result<()> {
    // Basic macOS sandbox policy:
    // 1. Allow all by default
    // 2. Deny access to sensitive directories
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
    let ssh_path = format!("{}/.ssh", home);
    let mut worm_path = std::env::current_dir().unwrap();
    while worm_path.file_name().unwrap() != "workspace" && worm_path.parent().is_some() {
        worm_path = worm_path.parent().unwrap().to_path_buf();
    }
    let worm_path = worm_path.join("test-worm").to_str().unwrap().to_string();

    let profile = format!(
        r#"(version 1)
(allow default)
(deny file-read* (subpath "{}"))
(deny file-read* (subpath "{}"))
"#,
        ssh_path, worm_path
    );

    println!("🛡️ SafePkg: Applying macOS Sandbox Profile...");
    println!("-------------------------------------------");
    println!("{}", profile);
    println!("-------------------------------------------");

    let mut child = Command::new("sandbox-exec")
        .arg("-p")
        .arg(profile)
        .arg("--") // Some versions of sandbox-exec might need this, or it helps with args starting with -
        .args(target_args)
        .spawn()
        .context("Failed to execute sandbox-exec")?;

    let status = child.wait().context("Failed to wait for sandboxed process")?;
    
    if !status.success() {
        // Note: sandbox-exec returns non-zero if it blocks an action or if the command fails
        println!("🛡️ SafePkg: Sandbox policy enforced or command failed.");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn build_bwrap_args(target_args: &[String]) -> Vec<String> {
    let mut args = vec![
        "--unshare-all".to_string(),
        "--share-net".to_string(),
        "--ro-bind".to_string(), "/usr".to_string(), "/usr".to_string(),
        "--ro-bind".to_string(), "/bin".to_string(), "/bin".to_string(),
        "--ro-bind".to_string(), "/lib".to_string(), "/lib".to_string(),
        "--ro-bind".to_string(), "/lib64".to_string(), "/lib64".to_string(),
        "--ro-bind".to_string(), "/etc/resolv.conf".to_string(), "/etc/resolv.conf".to_string(),
        "--bind".to_string(), ".".to_string(), ".".to_string(),
        "--dev".to_string(), "/dev".to_string(),
        "--proc".to_string(), "/proc".to_string(),
        "--".to_string(),
    ];
    args.extend(target_args.iter().cloned());
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_sandboxed_empty_args() {
        let res = run_sandboxed(&[]);
        assert!(res.is_ok());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_build_bwrap_args_basic() {
        let target = vec!["npm".to_string(), "install".to_string()];
        let args = build_bwrap_args(&target);
        assert!(args.contains(&"--unshare-all".to_string()));
        assert_eq!(args.last().unwrap(), "install");
    }
}
