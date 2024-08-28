use std::process::Command;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Command::new("pipenv")
    //     .arg("install")
    //     .status()
    //     .expect("Failed to install Python dependencies");
    // // Build Python executable with PyInstaller
    // Command::new("pipenv")
    //     .args(&["run", "pyinstaller", "--onefile", "your_script.py"])
    //     .status()
    //     .expect("Failed to create Python executable");

    // // Copy the Python executable to the output directory
    // fs::copy("dist/your_script", "target/debug/your_script")
    //     .expect("Failed to copy Python executable");

    let python_installed = Command::new("python3.9")
        .arg("--version")
        .output()
        .is_ok();

    if !python_installed {
        println!("Python 3.9 not found. Installing...");

        // Determine the current platform
        let target_os = env::consts::OS;
        match target_os {
            "linux" => install_python_linux(),
            "macos" => install_python_macos(),
            // Add support for other OSes if needed
            _ => panic!("Unsupported operating system"),
        }
    } else {
        println!("Python 3.9 is already installed.");
    }

    // 2. Install pipenv using the local Python binary
    let pipenv_install = Command::new("python3.9")
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("pipenv")
        .status()
        .expect("Failed to install pipenv");

    if !pipenv_install.success() {
        panic!("Failed to install pipenv.");
    }

    Command::new("pipenv")
        .arg("install")
        .status()
        .expect("Failed to install Python dependencies");

    println!("Successfully installed pipenv.");
}

fn install_python_linux() {
    // Install dependencies
    if !Command::new("sudo")
        .arg("apt-get")
        .arg("update")
        .status()
        .expect("Failed to update package list")
        .success()
    {
        panic!("Failed to update package list");
    }

    if !Command::new("sudo")
        .arg("apt-get")
        .arg("install")
        .arg("-y")
        .arg("software-properties-common")
        .status()
        .expect("Failed to install software-properties-common")
        .success()
    {
        panic!("Failed to install software-properties-common");
    }

    // Add deadsnakes PPA for Python 3.9
    if !Command::new("sudo")
        .arg("add-apt-repository")
        .arg("ppa:deadsnakes/ppa")
        .arg("-y")
        .status()
        .expect("Failed to add deadsnakes PPA")
        .success()
    {
        panic!("Failed to add deadsnakes PPA");
    }

    // Install Python 3.9
    if !Command::new("sudo")
        .arg("apt-get")
        .arg("install")
        .arg("-y")
        .arg("python3.9")
        .status()
        .expect("Failed to install Python 3.9")
        .success()
    {
        panic!("Failed to install Python 3.9");
    }

    println!("Python 3.9 installed successfully.");
}

fn install_python_macos() {
    // Check if Homebrew is installed
    if Command::new("brew")
        .arg("--version")
        .output()
        .is_err()
    {
        panic!("Homebrew is not installed. Please install it first.");
    }

    // Install Python 3.9 using Homebrew
    if !Command::new("brew")
        .arg("install")
        .arg("python@3.9")
        .status()
        .expect("Failed to install Python 3.9 with Homebrew")
        .success()
    {
        panic!("Failed to install Python 3.9 with Homebrew");
    }

    println!("Python 3.9 installed successfully.");
}
