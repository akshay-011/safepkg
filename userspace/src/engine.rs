use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub allow_list: AllowList,
}

#[derive(Debug, Deserialize, Default)]
pub struct AllowList {
    pub binaries: HashSet<String>,
}

pub struct SecurityEngine {
    config: Config,
}

impl SecurityEngine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Determines if a command execution is allowed based on the security policy.
    pub fn is_allowed(&self, command: &str) -> bool {
        // Basic heuristic: if it's in our allow list, it's fine.
        // In a real implementation, this would be much more sophisticated.
        if self.config.allow_list.binaries.contains(command) {
            return true;
        }

        // Potential malicious indicators
        let malicious_bins = ["curl", "wget", "bash", "sh", "nc", "python"];
        if malicious_bins.contains(&command) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_engine_scenarios() {
        let mut binaries = HashSet::new();
        binaries.insert("npm".to_string());
        binaries.insert("node".to_string());
        
        let config = Config {
            allow_list: AllowList { binaries },
        };
        let engine = SecurityEngine::new(config);

        println!("\n--- SafePkg Security Scenario Test ---");
        
        // Scenario 1: Trusted
        assert!(engine.is_allowed("npm"));
        println!("Checked 'npm': ALLOWED (Trusted)");

        // Scenario 2: Malware exfiltration
        assert!(!engine.is_allowed("curl"));
        println!("Checked 'curl': BLOCKED (High Risk)");

        // Scenario 3: Reverse shell
        assert!(!engine.is_allowed("nc"));
        println!("Checked 'nc': BLOCKED (Unauthorized Tool)");
        
        println!("--------------------------------------\n");
    }
}
