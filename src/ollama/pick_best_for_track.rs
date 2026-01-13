use color_eyre::eyre::{Context, OptionExt, Result};
use ollama_native::Ollama;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use crate::soulseek::{FileAttribute, SingleFileResult, Track};

#[derive(Serialize)]
struct FileResultRequest {
    file_id: i32,
    file_size: u64,
    filename: String,
    bitrate: Option<u32>,
    duration: Option<u32>,
}

#[derive(JsonSchema, Deserialize)]
struct Response {
    best_file_id: i32,
}

pub async fn pick_best_file_for_track(
    requested_track: Track,
    file_search_responses: &[SingleFileResult],
) -> Result<&SingleFileResult> {
    // TODO: only run if ollama is installed and running
    let ollama = Ollama::new("http://localhost:11434");

    let json_schema = schema_for!(Response);
    let json_schema_str = serde_json::to_string_pretty(&json_schema)
        .wrap_err("Failed to convert JSON schema to string")?;

    let file_searches = &file_search_responses
        .iter()
        .enumerate()
        .map(|(i, f)| FileResultRequest {
            file_id: i as i32,
            file_size: f.size,
            filename: f.filename.clone(),
            bitrate: f.attrs.get(&FileAttribute::Bitrate).map(|b| *b),
            duration: f.attrs.get(&FileAttribute::Duration).map(|d| *d),
        })
        .collect::<Vec<_>>();
    let file_search_responses_str = serde_json::to_string_pretty(&file_searches)
        .wrap_err("Failed to convert file search responses to string")?;

    let prompt = format!(
        r#"
You are a music file selection assistant. Your task is to choose the single best file from a list of file search responses, based on the user's request. The user provides a requested song title and may also specify an album, artist(s), and preferred duration (in seconds).

Instructions:

- Prefer high-definition (lossless or higher bitrate) audio files such as FLAC or high-bitrate MP3s when available.
- Ensure the chosen file matches the requested song title as closely as possible.
- If the user specifies an album, artist(s), or duration, prefer files that match those details.
- If the user requests a remix (the word "remix" is in the title or album), prefer files that are remixes and match the request.
- If the user does NOT request a remix, avoid files that are clearly labeled as remixes, edits, live, instrumental, karaoke, or cover versions.
- Rank files higher if they have additional information matching the user's request, such as correct duration, album, or artist tags in the filename or metadata.
- Return only the ID of the single best matching file in the specified JSON format.

Think step by step, explain what information from the request and the file responses you used to make your choice. Return your answer as JSON in the given schema.

User requested details:
track title: {title}
artists: {artists}
duration: {duration}

File search responses:
{file_search_responses}
"#,
        title = requested_track.title,
        artists = requested_track.artists.join(", "),
        duration = requested_track
            .length
            .map(|l| format!("{l} seconds"))
            .unwrap_or("not specified".to_string()),
        file_search_responses = file_search_responses_str
    );

    let response = ollama
        .generate("nemotron-3-nano:30b")
        .prompt(&prompt)
        .format(&json_schema_str)
        .await?;
    let response_json = serde_json::from_str::<Response>(&response.response)?;

    println!("structured JSON output:\n{}\n", response.response);
    let best_file_id = response_json.best_file_id;
    let best_file = file_searches
        .iter()
        .find(|&f| f.file_id == best_file_id)
        .ok_or_eyre("Best file not found")?;

    file_search_responses
        .iter()
        .find(|&f| f.filename == best_file.filename)
        .ok_or_eyre("Best file not found")
}
