use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[cfg(unix)]
fn main() {
    let wrappers_path = Path::new("./wrappers");

    let commands_dir = fs::read_dir("src/commands").unwrap();
    let commands = commands_dir.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();

        if let Some("rs") = path.extension().map(|os| os.to_str())? {
            path.file_stem()
                .map(|os| os.to_str().map(|s| s.to_string()))
                .flatten()
        } else {
            None
        }
    });

    let _ = fs::create_dir(&wrappers_path);

    for command in commands {
        let file_name = format!("git-{}", command);
        let file_path = wrappers_path.join(file_name);

        fs::write(
            &file_path,
            format!(
                "#!/bin/sh
gitbucket {}
",
                command
            ),
        )
        .unwrap();

        let metadata = fs::metadata(&file_path).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);

        fs::set_permissions(&file_path, permissions).unwrap();
    }

    println!("cargo:rerun-if-changed=src/commands/");
    println!("cargo:rerun-if-changed=build.rs");
}
