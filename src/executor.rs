/// Expand Windows-style %ENV_VAR% tokens in a string.
/// On non-Windows, also handles $VAR and ${VAR} for testing.
pub fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();

    // Windows-style %VAR%
    let mut output = String::with_capacity(result.len());
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' {
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '%') {
                let var_name: String = chars[i + 1..i + 1 + end].iter().collect();
                if let Ok(val) = std::env::var(&var_name) {
                    output.push_str(&val);
                } else {
                    // Leave unexpanded
                    output.push('%');
                    output.push_str(&var_name);
                    output.push('%');
                }
                i += end + 2;
                continue;
            }
        }
        output.push(chars[i]);
        i += 1;
    }
    result = output;

    result
}

/// Execute a command, optionally as Administrator (Windows ShellExecute "runas").
pub fn execute(cmd: &str, as_admin: bool) -> Result<(), String> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return Err("Nothing to run.".to_string());
    }

    #[cfg(windows)]
    {
        return execute_windows(cmd, as_admin);
    }

    #[cfg(not(windows))]
    {
        execute_unix(cmd, as_admin)
    }
}

#[cfg(windows)]
fn execute_windows(cmd: &str, as_admin: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Detect if it's a URL
    if cmd.starts_with("http://") || cmd.starts_with("https://") || cmd.starts_with("www.") {
        return open::that(cmd).map_err(|e| format!("Cannot open URL: {}", e));
    }

    let verb = if as_admin { "runas" } else { "open" };

    // Split into program and arguments
    let (program, args) = split_command(cmd);

    let verb_wide: Vec<u16> = OsStr::new(verb).encode_wide().chain(Some(0)).collect();
    let program_wide: Vec<u16> = OsStr::new(&program).encode_wide().chain(Some(0)).collect();
    let args_wide: Vec<u16> = OsStr::new(&args).encode_wide().chain(Some(0)).collect();

    let result = unsafe {
        winapi::um::shellapi::ShellExecuteW(
            std::ptr::null_mut(),
            verb_wide.as_ptr(),
            program_wide.as_ptr(),
            if args.is_empty() { std::ptr::null() } else { args_wide.as_ptr() },
            std::ptr::null(),
            winapi::um::winuser::SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW returns > 32 on success
    let code = result as usize;
    if code > 32 {
        Ok(())
    } else {
        let reason = match code {
            0  => "Out of memory or resources.",
            2  => "File not found.",
            3  => "Path not found.",
            5  => "Access denied.",
            8  => "Not enough memory.",
            26 => "Sharing violation.",
            27 => "Filename association incomplete or invalid.",
            28 => "DDE transaction timed out.",
            29 => "DDE transaction failed.",
            30 => "Another DDE transaction is being processed.",
            31 => "No application is associated with this file type.",
            32 => "The DLL could not be found.",
            _  => "Unknown error.",
        };
        Err(format!("Cannot run '{}': {}", program, reason))
    }
}

#[cfg(not(windows))]
fn execute_unix(cmd: &str, _as_admin: bool) -> Result<(), String> {
    // Try as URL first
    if cmd.starts_with("http://") || cmd.starts_with("https://") {
        return open::that(cmd).map_err(|e| format!("Cannot open URL: {}", e));
    }

    // Try as path
    let path = std::path::Path::new(cmd);
    if path.exists() {
        return open::that(path).map_err(|e| format!("Cannot open: {}", e));
    }

    // Try as shell command
    let (program, args_str) = split_command(cmd);
    let args: Vec<&str> = if args_str.is_empty() {
        vec![]
    } else {
        args_str.split_whitespace().collect()
    };

    std::process::Command::new(&program)
        .args(&args)
        .spawn()
        .map(|_| ())
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("'{}' was not found. Check the name and try again.", program)
            } else {
                format!("Cannot run '{}': {}", program, e)
            }
        })
}

/// Split "program args" respecting quoted program paths.
fn split_command(cmd: &str) -> (String, String) {
    let cmd = cmd.trim();
    if cmd.starts_with('"') {
        if let Some(end) = cmd[1..].find('"') {
            let program = cmd[1..end + 1].to_string();
            let args = cmd[end + 2..].trim().to_string();
            return (program, args);
        }
    }
    // No quotes: split on first space
    if let Some(pos) = cmd.find(' ') {
        (cmd[..pos].to_string(), cmd[pos + 1..].trim().to_string())
    } else {
        (cmd.to_string(), String::new())
    }
}
