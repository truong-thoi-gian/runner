use std::process::Command;
use std::io::{Write};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use chrono::Local;
use base64::encode;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Test cases data to encode
    let test_cases_data = r#"
create_issue:
  - open "https://truongthoigian.mantishub.io/login_page.php"
  - enter "ducnv+1@zotek8.com" in "username"
  - click "login"
  - ---
  - enter "Abcd1234" in "password"
  - click "login"
  - ---
  - click "Report Issue"
  - ---
  - select "[All Projects] General" from "Category"
  - select "major" from "Severity"
  - enter "test summary" in "Summary"
  - enter "test description" in "Description"
  - click "Submit Issue"
"#;

    // Encode the data to base64
    let encoded_data = encode(test_cases_data);

    // The ngrok URL
    let ngrok_url = "https://coherent-mostly-jawfish.ngrok-free.app"; // Replace with the actual ngrok URL you got

    // Create an HTTP client
    let client = Client::new();

    // Make the POST request
    let response = client.post(format!("{}/generate-script", ngrok_url))
        .form(&[("test_cases", &encoded_data)])
        .send()
        .await?;

    // Check if the response is successful
    if response.status().is_success() {
        // Parse the response as JSON
        let json_response: Value = response.json().await?;
        println!("{:?}", json_response);

        let subfolder = "tmp";
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        let file_name = format!("selenium_script_{}.py", timestamp);
        let file_path = Path::new(subfolder).join(file_name);

        // Ensure the subfolder exists
        fs::create_dir_all(subfolder)?;

        // Write the script to a .py file in the subfolder
        let mut file = File::create(&file_path)?;
        if let Some(script) = json_response.get("script") {
            file.write_all(script.as_str().unwrap_or("").as_bytes())?;
        }

        let additional_code = r#"
from selenium import webdriver
from selenium.common.exceptions import (NoAlertPresentException,
                                        UnexpectedAlertPresentException)
from selenium.webdriver.chrome.options import Options

options = Options()
# options.binary_location = Setting.BINARY_LOCATION
options.add_experimental_option("prefs", {"intl.accept_languages": "en_US"})
options.add_argument("--no-sandbox")
driver = webdriver.Chrome(options=options)

create_issue(driver)
"#;

        let mut file = OpenOptions::new()
            .append(true)
            .open(&file_path)?;
        file.write_all(additional_code.as_bytes())?;

        // Execute the script using Pipenv
        let output = Command::new("pipenv")
            .arg("run")
            .arg("python")
            .arg(&file_path)
            .current_dir("./") // Set the working directory to where your Pipenv environment is
            .output()?; // Execute the command and get the output

        // Check if the command was successful
        if output.status.success() {
            println!("Script executed successfully");
        } else {
            // Print the error message if the script failed
            eprintln!(
                "Error: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("The script has been successfully exported to {:?}", file_path);
    } else {
        eprintln!("Request failed with status: {}", response.status());
    }

    Ok(())
}
