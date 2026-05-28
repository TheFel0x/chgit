use std::collections::HashMap;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Add a new git profile
    Add {
        profile_name: String,
        user_name: String,
        email: String,
        ssh_key: String,
    },
    /// Remove a saved git profile
    Remove { profile_name: String },
    /// List all saved git profiles
    List,
    /// Switch to a saved git profile
    Switch { profile_name: String },
    /// Set auto profile selection for a path. If not path is specified the working directory is used
    SetAuto {
        profile_name: String,
        path: Option<String>,
    },
    /// Auto select profile for current path
    Auto,
    /// Checks which profile is currently active (if any applies)
    CheckProfile,
    /// Add the current git config as a profile with the given name
    AddCurrent { profile_name: String },
    /// Checks the current local git configuration and prints it
    CheckGitConfig,
}

#[derive(Serialize, Deserialize)]
struct Profile {
    user_name: String,
    email: String,
    ssh_key: String,
}

#[derive(Serialize, Deserialize)]
struct Config {
    /// Saved profiles, keyed by profile name
    profiles: HashMap<String, Profile>,
    /// Maps path → profile name for auto profile selection
    auto_paths: HashMap<String, String>,
}

impl Config {
    /// Get the path to the config file
    fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .expect("could not find config directory")
            .join("gp")
            .join("config.toml")
    }

    /// Load the config from the config file. If the file does not exist or is invalid, return an empty config
    fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self {
                profiles: HashMap::new(),
                auto_paths: HashMap::new(),
            };
        }
        let contents = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => {
                return Self {
                    profiles: HashMap::new(),
                    auto_paths: HashMap::new(),
                }
            }
        };
        toml::from_str(&contents).expect("config file is invalid")
    }

    /// Save the config to the config file
    fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create config directory");
        }
        let contents = toml::to_string(self).expect("failed to serialize config");
        std::fs::write(path, contents).expect("failed to write config file");
    }
}

fn stdio(verbose: bool) -> std::process::Stdio {
    if verbose {
        std::process::Stdio::inherit()
    } else {
        std::process::Stdio::null()
    }
}

fn main() {
    let cli = Cli::parse();
    let mut config = Config::load();

    match cli.command {
        Command::Add {
            profile_name,
            user_name,
            email,
            ssh_key,
        } => {
            let new_profile = Profile {
                user_name,
                email,
                ssh_key,
            };
            config.profiles.insert(profile_name, new_profile);
            config.save();
        }
        Command::Remove { profile_name } => {
            config.profiles.remove(&profile_name);
            config.save();
        }
        Command::List => {
            for (name, profile) in config.profiles {
                println!("Profile: {name}");
                println!("  User Name: {}", profile.user_name); // TODO: mask user name
                println!("  Email: {}", profile.email); // TODO: mask email
                println!("  SSH Key: {}", profile.ssh_key);
            }
        }
        Command::Switch { profile_name } => {
            if let Some(profile) = config.profiles.get(&profile_name) {
                // Call git config
                println!("switching to profile: {profile_name}");
                apply_profile(profile, cli.verbose);
            } else {
                println!("profile not found: {profile_name}");
            }
        }
        Command::SetAuto { profile_name, path } => {
            let path = path.unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            });
            if config.profiles.contains_key(&profile_name) {
                config.auto_paths.insert(path, profile_name);
                config.save();
            } else {
                println!("profile not found: {profile_name}");
            }
        }
        Command::Auto => {
            let path = std::env::current_dir().unwrap();
            let mut current_path = Some(path.as_path());
            while let Some(p) = current_path {
                if let Some(profile_name) = config.auto_paths.get(p.to_string_lossy().as_ref()) {
                    if let Some(profile) = config.profiles.get(profile_name) {
                        println!("auto switching to profile: {profile_name}");
                        apply_profile(profile, cli.verbose);
                        return;
                    }
                }
                current_path = p.parent();
            }
        }
        Command::CheckProfile => {
            let active = get_git_local(cli.verbose);
            if active.is_none() {
                println!("no active profile found");
            } else {
                for (name, profile) in config.profiles {
                    if profile.user_name == active.as_ref().unwrap().user_name
                        && profile.email.to_lowercase()
                            == active.as_ref().unwrap().email.to_lowercase()
                    //&& profile.ssh_key == active.as_ref().unwrap().ssh_key // TODO: Make checking the ssh key work
                    {
                        println!("active profile: {name}");
                        return;
                    }
                }
                println!("active configuration not stored in gp");
            }
        }
        Command::AddCurrent { profile_name } => {
            let active = get_git_local(cli.verbose);
            if active.is_none() {
                println!("no active configuration found, cannot add as profile");
            } else {
                let profile = active.unwrap();
                config.profiles.insert(profile_name, profile);
                config.save();
            }
        }
        Command::CheckGitConfig => {
            let active = get_git_local(cli.verbose);
            if active.is_none() {
                let global = get_git_global(cli.verbose);
                if global.is_none() {
                    println!("no active local, nor global configuration found");
                } else {
                    let profile = global.unwrap();
                    println!("active global configuration:");
                    println!("  user.name: {}", profile.user_name);
                    println!("  user.email: {}", profile.email);
                    println!("  ssh key: {}", profile.ssh_key);
                }
            } else {
                let profile = active.unwrap();
                println!("active local configuration:");
                println!("  user.name: {}", profile.user_name);
                println!("  user.email: {}", profile.email);
                println!("  ssh key: {}", profile.ssh_key);
            }
        }
    }
}

fn apply_profile(profile: &Profile, verbose: bool) {
    // Call git config
    std::process::Command::new("git")
        .args(["config", "--local", "user.name", &profile.user_name])
        .stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .status()
        .expect("failed to run git");
    std::process::Command::new("git")
        .args(["config", "--local", "user.email", &profile.email])
        .stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .status()
        .expect("failed to run git");
    // Call ssh-add
    std::process::Command::new("ssh-add")
        .arg(&profile.ssh_key)
        .stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .status()
        .expect("failed to run ssh-add");
}

/// Check the current git local configuration and return a profile corresponding to the user.name, user.email and ssh key
/// This happens regardless of the configuration being stored in gp or not
fn get_git_local(verbose: bool) -> Option<Profile> {
    // user.name
    let user_name_output = std::process::Command::new("git")
        .args(["config", "--local", "user.name"])
        //.stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !user_name_output.status.success() {
        return None;
    }
    let user_name = String::from_utf8_lossy(&user_name_output.stdout)
        .trim()
        .to_string();

    // user.email
    let email_output = std::process::Command::new("git")
        .args(["config", "--local", "user.email"])
        //.stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !email_output.status.success() {
        return None;
    }
    let email = String::from_utf8_lossy(&email_output.stdout)
        .trim()
        .to_string();

    // ssh key
    let ssh_key_output = std::process::Command::new("ssh-add")
        .args(["-L"])
        //.stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run ssh-add");
    if !ssh_key_output.status.success() {
        return None;
    }
    let ssh_key = String::from_utf8_lossy(&ssh_key_output.stdout)
        .trim()
        .to_string();

    return Some(Profile {
        user_name,
        email,
        ssh_key,
    });
}

/// Fallback for CheckGitConfig command to display global if needed
fn get_git_global(verbose: bool) -> Option<Profile> {
    // user.name
    let user_name_output = std::process::Command::new("git")
        .args(["config", "--global", "user.name"])
        //.stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !user_name_output.status.success() {
        return None;
    }
    let user_name = String::from_utf8_lossy(&user_name_output.stdout)
        .trim()
        .to_string();

    // user.email
    let email_output = std::process::Command::new("git")
        .args(["config", "--global", "user.email"])
        //.stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !email_output.status.success() {
        return None;
    }
    let email = String::from_utf8_lossy(&email_output.stdout)
        .trim()
        .to_string();

    // ssh key
    let ssh_key_output = std::process::Command::new("ssh-add")
        .args(["-L"])
        //.stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run ssh-add");
    if !ssh_key_output.status.success() {
        return None;
    }
    let ssh_key = String::from_utf8_lossy(&ssh_key_output.stdout)
        .trim()
        .to_string();

    return Some(Profile {
        user_name,
        email,
        ssh_key,
    });
}
