use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[cfg(unix)]
fn main() {
    let mut commands = HashSet::new();
    commands.extend(read_commands_in_path("src/commands"));
    commands.extend(read_commands_in_path("src/gitbucket/commands"));
    commands.extend(read_commands_in_path("src/gitlad/commands"));

    let wrappers_path = Path::new("./wrappers");

    let _ = fs::remove_dir(&wrappers_path);
    let _ = fs::create_dir(&wrappers_path);

    for command in commands {
        let file_name = format!("git-{}", command);
        let file_path = wrappers_path.join(file_name);

        fs::write(
            &file_path,
            format!(
                "#!/bin/sh
gitext {} $@
",
                command
            ),
        )
        .unwrap();
    }

    println!("cargo:rerun-if-changed=src/commands/");
    println!("cargo:rerun-if-changed=build.rs");
}

fn read_commands_in_path(path: &str) -> impl Iterator<Item = String> {
    let commands_dir = fs::read_dir(path).unwrap();
    commands_dir.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();

        if let Some("rs") = path.extension().map(|os| os.to_str())? {
            path.file_stem()
                .map(|os| os.to_str().map(|s| s.to_string()))
                .flatten()
        } else {
            None
        }
    })
}
