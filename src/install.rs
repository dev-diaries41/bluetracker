use std::process::Command;

pub fn check_dependencies() -> bool {
    let dependencies = vec!["sqlite3"];
    let mut all_found = true;

    for dep in dependencies {
        let output = Command::new("which")
            .arg(dep)
            .output();

        if let Ok(output) = output {
            if !output.status.success() {
                println!("Missing dependency: {}", dep);
                all_found = false;
            }
        } else {
            println!("Failed to check dependency: {}", dep);
            all_found = false;
        }
    }

    if all_found {
        println!("All dependencies are installed.");
    } else {
        println!("Some dependencies are missing. Please install them.");
    }

    all_found
}
