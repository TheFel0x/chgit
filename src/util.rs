/// `john.doe@example.com` -> `j******e@e*****e.com`
pub fn censor_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return "*".repeat(email.len());
    }
    let local = parts[0];
    let domain = parts[1];
    let censored_local = if local.len() <= 2 {
        "*".repeat(local.len())
    } else {
        format!(
            "{}{}{}",
            &local[0..1],
            "*".repeat(local.len() - 2),
            &local[local.len() - 1..]
        )
    };
    let censored_domain = if domain.len() <= 2 {
        "*".repeat(domain.len())
    } else {
        format!(
            "{}{}{}",
            &domain[0..1],
            "*".repeat(domain.len() - 2),
            &domain[domain.len() - 1..]
        )
    };
    format!("{}@{}", censored_local, censored_domain)
}

/// `John Doe` -> `J***_**e`
pub fn censor_name(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    let censored_parts: Vec<String> = parts
        .iter()
        .map(|part| {
            if part.len() <= 2 {
                "*".repeat(part.len())
            } else {
                format!(
                    "{}{}{}",
                    &part[0..1],
                    "*".repeat(part.len() - 2),
                    &part[part.len() - 1..]
                )
            }
        })
        .collect();
    censored_parts.join(" ")
}

/// C:\Users\JohnDoe\.ssh\id_rsa -> C:\Users\J*******\.ssh\i*_**a
/// ~/.ssh/id_rsa -> ~/.ssh/i*_**a
/// /home/john/.ssh/id_rsa -> /home/j***/.ssh/i*_**a
pub fn censor_ssh_key(ssh_key: &str) -> String {
    let parts: Vec<&str> = ssh_key.rsplitn(2, std::path::MAIN_SEPARATOR).collect();
    if parts.len() != 2 {
        return "*".repeat(ssh_key.len());
    }
    let file_name = parts[0];
    let dir = parts[1];
    let censored_file_name = if file_name.len() <= 2 {
        "*".repeat(file_name.len())
    } else {
        format!(
            "{}{}{}",
            &file_name[0..1],
            "*".repeat(file_name.len() - 2),
            &file_name[file_name.len() - 1..]
        )
    };
    let censored_dir = if dir.len() <= 2 {
        "*".repeat(dir.len())
    } else {
        format!(
            "{}{}{}",
            &dir[0..1],
            "*".repeat(dir.len() - 2),
            &dir[dir.len() - 1..]
        )
    };
    format!(
        "{}{}{}",
        censored_dir,
        std::path::MAIN_SEPARATOR,
        censored_file_name
    )
}
