[![Build & Test](https://github.com/TheFel0x/chgit/actions/workflows/rust.yml/badge.svg)](https://github.com/TheFel0x/chgit/actions/workflows/rust.yml)

# chgit

Quickly switch between your identities, without having to deal with git config files.

## SSH-Agent Setup

### Windows

The OpenSSH agent might be disabled by default on Windows. Run the following commands in an **elevated (Administrator) PowerShell**:

```powershell
# Allow ssh-agent to be started
Set-Service ssh-agent -StartupType Manual  
# Start ssh-agent
Start-Service ssh-agent
# Automatically start ssh-agent at boot from now on
Set-Service ssh-agent -StartupType Automatic 
```

If all three commands are executed this is a one time setup and can be skipped in the future. 

### Linux

Ensure your `ssh-agent` is running.

```bash
eval "$(ssh-agent -s)"
```

Consider adding this to your shell's startup file (e.g., `~/.bashrc` or `~/.zshrc`) or your profile script.

### macOS

Should be the same as Linux? Not sure, I don't have a macOS machine to test this.

## Basic Usage

1. Create a new profile:

```bash
chgit add <PROFILE_NAME> <USER_NAME> <EMAIL> <SSH_KEY> [--gpg-key <GPG_KEY>]
```
Examples:

```bash
# Linux example
chgit add Private "John Doe" "john.doe@example.com" "/home/john/.ssh/id_rsa_work"
# Linux example with GPG signing
chgit add Private "John Doe" "john.doe@example.com" "/home/john/.ssh/id_rsa_work" --gpg-key ABC1234567
# Windows example
chgit.exe add Private "John Doe" "john.doe@example.com" "C:\Users\John\.ssh\id_rsa_work"
```

2. List profiles:

```bash
chgit list
```

3. Switch to a profile:

```bash
chgit switch <PROFILE_NAME>
```

## Autopaths

1. Add an autopath:

```bash
chgit set-auto <PROFILE_NAME> [PATH]
```

If no path is provided, the current working directory will be used.

2. Auto switch depending on the current path:

```bash
chgit auto
```

## Other Commands

- `chgit remove <PROFILE_NAME>`: Remove a profile.
- `chgit info <PROFILE_NAME>`: Show full details of a profile.
- `chgit check-profile`: Check the active profile. (if any)
- `chgit add-current <PROFILE_NAME>`: Create a new profile with the current git config values.
- `chgit check-git-config`: Quickly check the current git config values.
- `chgit unset`: Unset the local git name and email.
- `chgit help`: Show the help message.
