use std::process::Command;

fn main() {
    let _build_command = Command::new("sh")
        .arg("script/build/build_project.sh")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}