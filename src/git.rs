use crate::Profile;

pub fn stdio(verbose: bool) -> std::process::Stdio {
    if verbose {
        std::process::Stdio::inherit()
    } else {
        std::process::Stdio::null()
    }
}

pub fn apply_profile(profile: &Profile, verbose: bool) {
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
    // Configure GPG signing if a key is set
    if let Some(gpg_key) = &profile.gpg_key {
        std::process::Command::new("git")
            .args(["config", "--local", "user.signingkey", gpg_key])
            .stdout(stdio(verbose))
            .stderr(stdio(verbose))
            .status()
            .expect("failed to run git");
        std::process::Command::new("git")
            .args(["config", "--local", "commit.gpgsign", "true"])
            .stdout(stdio(verbose))
            .stderr(stdio(verbose))
            .status()
            .expect("failed to run git");
    } else {
        // If no GPG key is set, disable signing
        std::process::Command::new("git")
            .args(["config", "--local", "commit.gpgsign", "false"])
            .stdout(stdio(verbose))
            .stderr(stdio(verbose))
            .status()
            .expect("failed to run git");
    }
    // Call ssh-add
    std::process::Command::new("ssh-add")
        .arg(&profile.ssh_key)
        .stdout(stdio(verbose))
        .stderr(stdio(verbose))
        .status()
        .expect("failed to run ssh-add");
}

/// Read the local git user.name, user.email and loaded SSH key
pub fn get_git_local(verbose: bool) -> Option<Profile> {
    // user.name
    let user_name_output = std::process::Command::new("git")
        .args(["config", "--local", "user.name"])
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !user_name_output.status.success() {
        if verbose {
            println!("no local user.name found");
        }
        return None;
    }
    let user_name = String::from_utf8_lossy(&user_name_output.stdout)
        .trim()
        .to_string();

    // user.email
    let email_output = std::process::Command::new("git")
        .args(["config", "--local", "user.email"])
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !email_output.status.success() {
        if verbose {
            println!("no local user.email found");
        }
        return None;
    }
    let email = String::from_utf8_lossy(&email_output.stdout)
        .trim()
        .to_string();

    // ssh key
    let ssh_key_output = std::process::Command::new("ssh-add")
        .args(["-L"])
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run ssh-add");
    if !ssh_key_output.status.success() {
        if verbose {
            println!("no local ssh key found");
        }
        return None;
    }
    let ssh_key = String::from_utf8_lossy(&ssh_key_output.stdout)
        .trim()
        .to_string();

    Some(Profile {
        user_name,
        email,
        ssh_key,
        gpg_key: None,
    })
}

/// Read the global git user.name, user.email and loaded SSH key
pub fn get_git_global(verbose: bool) -> Option<Profile> {
    // user.name
    let user_name_output = std::process::Command::new("git")
        .args(["config", "--global", "user.name"])
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !user_name_output.status.success() {
        if verbose {
            println!("no global user.name found");
        }
        return None;
    }
    let user_name = String::from_utf8_lossy(&user_name_output.stdout)
        .trim()
        .to_string();

    // user.email
    let email_output = std::process::Command::new("git")
        .args(["config", "--global", "user.email"])
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run git");
    if !email_output.status.success() {
        if verbose {
            println!("no global user.email found");
        }
        return None;
    }
    let email = String::from_utf8_lossy(&email_output.stdout)
        .trim()
        .to_string();

    // ssh key
    let ssh_key_output = std::process::Command::new("ssh-add")
        .args(["-L"])
        .stderr(stdio(verbose))
        .output()
        .expect("failed to run ssh-add");
    if !ssh_key_output.status.success() {
        if verbose {
            println!("no global ssh key found");
        }
        return None;
    }
    let ssh_key = String::from_utf8_lossy(&ssh_key_output.stdout)
        .trim()
        .to_string();

    Some(Profile {
        user_name,
        email,
        ssh_key,
        gpg_key: None,
    })
}
