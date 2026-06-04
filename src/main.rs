mod git;
mod util;

use std::collections::HashMap;

use clap::{Parser, Subcommand};
use git::{apply_profile, get_git_global, get_git_local, stdio};

use serde::{Deserialize, Serialize};
use util::{censor_email, censor_name, censor_ssh_key};

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
    /// Shows full information about the specified profile
    Info { profile_name: String },
    /// Unsets local name, email and ssh key
    Unset,
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
            .join("chgit")
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
                if cli.verbose {
                    println!("  User Name: {}", censor_name(&profile.user_name));
                    println!("  Email: {}", censor_email(&profile.email));
                    println!("  SSH Key: {}", censor_ssh_key(&profile.ssh_key));
                }
            }
        }
        Command::Switch { profile_name } => {
            if let Some(profile) = config.profiles.get(&profile_name) {
                // Call git config
                if cli.verbose {
                    println!("switching to profile: {profile_name}");
                }
                apply_profile(profile, cli.verbose);
            } else {
                if cli.verbose {
                    println!("profile not found: {profile_name}");
                }
                std::process::exit(1);
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
                if cli.verbose {
                    println!("profile not found: {profile_name}");
                }
                std::process::exit(1);
            }
        }
        Command::Auto => {
            let path = std::env::current_dir().unwrap();
            let mut current_path = Some(path.as_path());
            while let Some(p) = current_path {
                if let Some(profile_name) = config.auto_paths.get(p.to_string_lossy().as_ref()) {
                    if let Some(profile) = config.profiles.get(profile_name) {
                        if cli.verbose {
                            println!("auto switching to profile: {}", profile_name);
                        }
                        apply_profile(profile, cli.verbose);
                        return;
                    }
                }
                current_path = p.parent();
            }
            if cli.verbose {
                println!("no auto profile found for current path");
            }
            std::process::exit(1);
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
                if cli.verbose {
                    println!("active configuration not stored in gp");
                }
                std::process::exit(1);
            }
        }
        Command::AddCurrent { profile_name } => {
            let active = get_git_local(cli.verbose);
            if active.is_none() {
                if cli.verbose {
                    println!("no active configuration found, cannot add as profile");
                }
                std::process::exit(1);
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
                    if cli.verbose {
                        println!("no active local, nor global configuration found");
                    }
                    std::process::exit(1);
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
        Command::Info { profile_name } => {
            if let Some(profile) = config.profiles.get(&profile_name) {
                println!("Profile: {profile_name}");
                println!("  User Name: {}", profile.user_name); // TODO: mask user name
                println!("  Email: {}", profile.email); // TODO: mask email
                println!("  SSH Key: {}", profile.ssh_key);
            } else {
                if cli.verbose {
                    println!("profile not found: {profile_name}");
                }
                std::process::exit(1);
            }
        }
        Command::Unset => {
            // Unset local user.name, user.email and ssh key
            std::process::Command::new("git")
                .args(["config", "--local", "--unset", "user.name"])
                .stdout(stdio(cli.verbose))
                .stderr(stdio(cli.verbose))
                .status()
                .expect("failed to run git");
            std::process::Command::new("git")
                .args(["config", "--local", "--unset", "user.email"])
                .stdout(stdio(cli.verbose))
                .stderr(stdio(cli.verbose))
                .status()
                .expect("failed to run git");
        }
    }
}

#[cfg(test)]
mod tests;
