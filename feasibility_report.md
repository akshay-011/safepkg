# Feasibility Report & Architecture Design: Dynamic Sandbox for Package Installation

## 1. Executive Summary
> ⚠️ **Note**: This project is **vibe coded**. Please be aware of this development style and its implications.

The proposed product aims to secure the software supply chain by providing a **dynamic sandbox** for package installation (`npm install`, `yarn install`, etc.).
 Unlike existing solutions that rely on static databases, this tool will monitor the behavior of install scripts in real-time, detecting and blocking malicious actions (e.g., data exfiltration, sensitive file access) by analyzing process parentage and system call patterns.

## 2. Feasibility Analysis

### 2.1 Technical Feasibility: HIGH (Linux), MEDIUM (macOS)
*   **Linux**: Highly feasible using **Linux Namespaces** (via `bubblewrap`) for sandboxing and **eBPF** for real-time monitoring and blocking. This combination allows for lightweight, fine-grained control without requiring root for the end-user (in many distributions).
*   **macOS**: Medium feasibility. Apple's **Endpoint Security (ES) framework** allows monitoring, but creating a restricted sandbox for a single command is more complex due to SIP and the deprecation of `sandbox-exec`. A Docker-based fallback or a more restrictive ES-based monitor is required.
*   **Performance**: Overhead of eBPF and Namespaces is negligible for the duration of a package installation.

### 2.2 Uniqueness & Value Proposition
Existing tools like `safe-chain` or `socket.dev` primarily use **Static Analysis** or **Proxy-based Threat Intel**.
This proposal adds **Dynamic Runtime Analysis**, which catches:
1.  **Zero-day malware**: Packages not yet in any threat database.
2.  **Obfuscated scripts**: Malicious code that hides its intent until execution.
3.  **Environment-specific attacks**: Malware that only triggers in certain CI/CD or dev environments.

## 3. Proposed Architecture

### 3.1 High-Level Components
1.  **`safe-pkg` CLI**: A wrapper that intercepts package manager commands.
2.  **Sandbox Orchestrator**: Sets up the isolated environment (Filesystem, Network, Process space).
3.  **Behavioral Monitor (eBPF)**: A kernel-level observer that tracks every process spawned within the sandbox.
4.  **Rule Engine**: Validates process activity against a "Least Privilege" policy.
5.  **Alert/Kill Switch**: Immediately terminates the sandbox if a violation occurs.

### 3.2 Sandboxing Strategy (Linux)
*   **Tool**: `bubblewrap` (bwrap).
*   **Filesystem**:
    *   Mount `node_modules` and project files as read-write.
    *   Mount system binaries (`/usr/bin`, `/lib`) as read-only.
    *   **Deny access** to `~/.ssh`, `~/.env`, `/etc/shadow`, etc.
*   **Network**:
    *   Allow traffic only to known registry domains (npm, PyPI) and GitHub (for git dependencies).
    *   Block all other outbound traffic.

### 3.3 Monitoring Strategy (eBPF)
*   **Event Hooks**: `sys_enter_execve` (process creation), `sys_enter_connect` (network calls), `sys_enter_openat` (file access).
*   **Parentage Tracking**: Every event is tagged with its PID and PPID. The monitor builds a tree starting from the package manager.
*   **Heuristics**:
    *   Flag if `node` spawns `curl`, `wget`, or `bash` to download external scripts.
    *   Flag if any child process attempts to read files outside the project directory.

## 4. Recommended Tech Stack

| Component | Technology | Reasoning |
| :--- | :--- | :--- |
| **Language** | **Rust** | Best-in-class performance and memory safety. Excellent eBPF support via **Aya**. |
| **eBPF Library** | **Aya** | Allows writing both userspace and kernel-space (eBPF) code in 100% Rust. No C dependencies. |
| **Sandboxing** | **Bubblewrap** | Industry standard for unprivileged sandboxing on Linux. Easily invoked via Rust's `std::process`. |
| **Monitoring** | **eBPF (CO-RE)** | Allows for portable, high-performance monitoring across different Linux kernel versions. |
| **Configuration** | **YAML/TOML** | Standard for defining behavioral rules. Rust has excellent `serde` support. |

### Why Rust is actually BETTER for this:
1.  **Aya (eBPF in Rust)**: Unlike Go (which often requires a C compiler and LLVM to compile eBPF programs), Aya allows you to write eBPF programs in Rust and compile them using the standard Rust toolchain. This simplifies the build process significantly.
2.  **Zero-Cost Abstractions**: Monitoring system calls requires low overhead. Rust provides this without the garbage collection pauses that can occur in Go.
3.  **Security**: Since this is a security tool, Rust's memory safety prevents common vulnerabilities (buffer overflows, etc.) that could be exploited to bypass the sandbox.
4.  **Static Binaries**: Rust produces small, self-contained binaries that are easy to distribute as a single file, similar to Go.

## 5. Implementation Roadmap (MVP)
1.  **Phase 1 (Sandbox)**: Build a Rust CLI that wraps `npm install` in a `bubblewrap` container with restricted FS access using `std::process`.
2.  **Phase 2 (Monitoring)**: Integrate a basic **Aya** eBPF program to log all `execve` calls within the sandbox.
3.  **Phase 3 (Enforcement)**: Implement the Rule Engine in Rust to block non-essential outbound connections.
4.  **Phase 4 (Reporting)**: Create a dashboard or CLI report showing the "Behavioral Profile" of the installed packages.

## 6. Challenges & Mitigations
*   **Complexity of `node-gyp`**: Some packages compile C++ during install.
    *   *Mitigation*: Pre-define a "Compiler Allow-list" (gcc, g++, make).
*   **OS Portability**: eBPF is Linux-only.
    *   *Mitigation*: Use a plugin-based architecture where the Linux driver uses eBPF and the macOS driver uses Endpoint Security.
