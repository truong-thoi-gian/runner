use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use chrono::Local;
use base64::encode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::Filter;
use std::error::Error;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
struct FormData {
    test_cases: String,
}

#[derive(Debug)]
struct CustomError(String);

impl warp::reject::Reject for CustomError {}

async fn handle_form_submission(form_data: FormData) -> Result<String, warp::Rejection> {
    let test_cases_data = form_data.test_cases;

    // Encode the data to base64
    let encoded_data = encode(&test_cases_data);

    // The ngrok URL
    let ngrok_url = "https://coherent-mostly-jawfish.ngrok-free.app"; // Replace with the actual ngrok URL you got

    // Create an HTTP client
    let client = Client::new();

    // Make the POST request
    let response = client.post(format!("{}/generate-script", ngrok_url))
        .form(&[("test_cases", &encoded_data)])
        .send()
        .await
        .map_err(|_| warp::reject::custom(CustomError("Request failed".into())))?;

    if response.status().is_success() {
        // Parse the response as JSON
        let json_response: Value = response.json().await.map_err(|_| warp::reject::custom(CustomError("Failed to parse JSON".into())))?;

        let subfolder = "tmp";
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        let file_name = format!("selenium_script_{}.py", timestamp);
        let file_path = Path::new(subfolder).join(file_name);

        // Ensure the subfolder exists
        fs::create_dir_all(subfolder).map_err(|_| warp::reject::custom(CustomError("Failed to create directory".into())))?;

        // Write the script to a .py file in the subfolder
        let mut file = File::create(&file_path).map_err(|_| warp::reject::custom(CustomError("Failed to create file".into())))?;
        if let Some(script) = json_response.get("script") {
            file.write_all(script.as_str().unwrap_or("").as_bytes()).map_err(|_| warp::reject::custom(CustomError("Failed to write to file".into())))?;
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
            .open(&file_path)
            .map_err(|_| warp::reject::custom(CustomError("Failed to open file for appending".into())))?;
        file.write_all(additional_code.as_bytes()).map_err(|_| warp::reject::custom(CustomError("Failed to write additional code".into())))?;

        // Execute the script using Pipenv
        let output = Command::new("pipenv")
            .arg("run")
            .arg("python")
            .arg(&file_path)
            .current_dir("./") // Set the working directory to where your Pipenv environment is
            .output()
            .await
            .map_err(|_| warp::reject::custom(CustomError("Failed to execute command".into())))?;

        // Check if the command was successful
        if output.status.success() {
            Ok(format!("The script has been successfully exported to {:?}", file_path))
        } else {
            Err(warp::reject::custom(CustomError(format!(
                "Error: {}",
                String::from_utf8_lossy(&output.stderr)
            ))))
        }
    } else {
        Err(warp::reject::custom(CustomError(format!(
            "Request failed with status: {}",
            response.status()
        ))))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Define the HTML form as a static string
    let form_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Cases Form</title>
        </head>
        <body>
            <h1>Enter Test Cases Data</h1>
            <form action="/submit" method="post">
                <textarea name="test_cases" rows="20" cols="80"></textarea><br>
                <input type="submit" value="Submit">
            </form>
        </body>
        </html>
    "#;

    // Define the route for serving the HTML form
    let form_route = warp::get()
        .and(warp::path::end())
        .map(move || warp::reply::html(form_html));

    // Define the route for the form submission
    let submit_route = warp::post()
        .and(warp::path("submit"))
        .and(warp::body::form())
        .and_then(handle_form_submission);

    // Combine routes
    let routes = form_route.or(submit_route);

    // Start the warp server
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
