#![warn(clippy::all, clippy::pedantic)]

use color_eyre::Result;
use ollama_rs::Ollama;
use serde::Deserialize;

/// Response structure for /api/ps endpoint (running models)
#[derive(Deserialize)]
struct RunningModelsResponse {
    models: Vec<RunningModel>,
}

/// A model that is currently loaded in memory
#[derive(Deserialize)]
struct RunningModel {
    /// The name of the running model (e.g., "qwen3-coder:latest")
    model: String,
}

/// Check if Ollama API is running by calling GET /api/tags
/// Returns Ok with list of model names if Ollama is reachable
/// Uses a 5 second timeout
/// # Errors
/// Will error with message if Ollama is not reachable
pub async fn check_ollama_running() -> Result<Vec<String>, String> {
    let ollama = Ollama::default();

    // Use tokio timeout to limit the check to 5 seconds
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ollama.list_local_models(),
    )
    .await;

    match result {
        Ok(Ok(models)) => {
            // Extract model names from the response
            let model_names: Vec<String> = models.into_iter().map(|m| m.name).collect();
            Ok(model_names)
        }
        Ok(Err(_)) | Err(_) => {
            Err("Error: Ollama is not running. Please start Ollama and try again.".to_string())
        }
    }
}

/// Check if the configured model is installed
/// Returns Ok(()) if model is found in the list
/// # Errors
/// Will error if model is not present
pub fn check_model_installed(model_name: &str, installed_models: &[String]) -> Result<(), String> {
    // Check if the model name appears in the installed models list
    // Model names in Ollama can include tags (e.g., "llama2:latest")
    if installed_models.iter().any(|m| m == model_name) {
        Ok(())
    } else {
        Err(format!(
            "Error: Model {model_name} is not installed. Run `ollama pull {model_name}` to install it."
        ))
    }
}

/// Check if the model is currently loaded in memory by calling GET /api/ps.
/// Returns Ok(true) if model is loaded, Ok(false) if not loaded,
/// # Errors
/// Will error on connection error.
pub async fn is_model_loaded(model_name: &str) -> Result<bool, String> {
    let ollama = Ollama::default();
    let url = format!("{}api/ps", ollama.url_str());

    // Use a simple HTTP client from reqwest (via ollama-rs internal client pattern)
    let client = reqwest::Client::new();
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), client.get(&url).send())
        .await
        .map_err(|_| "Timeout checking running models".to_string())?
        .map_err(|e| format!("Failed to check running models: {e}"))?;

    if !result.status().is_success() {
        return Err("Failed to get running models".to_string());
    }

    let response: RunningModelsResponse = result
        .json()
        .await
        .map_err(|e| format!("Failed to parse running models response: {e}"))?;

    // Check if our model is in the running models list
    Ok(response.models.iter().any(|m| m.model == model_name))
}
