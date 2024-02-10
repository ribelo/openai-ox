use reqwest::multipart;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

use crate::{ApiRequestError, ErrorResponse, OpenAi, BASE_URL};

const API_URL: &str = "v1/audio/transcriptions";

#[derive(Debug)]
pub enum AudioFormat {
    Mp3,
    Mp4,
    Flac,
    Mpeg,
    Mpga,
    M4a,
    Ogg,
    Wav,
    Webm,
}

impl AudioFormat {
    pub fn to_mime(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Mp4 => "audio/mp4",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Mpeg => "audio/mpeg",
            AudioFormat::Mpga => "audio/mpeg",
            AudioFormat::M4a => "audio/mp4",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Webm => "audio/webm",
        }
    }
    pub fn to_extension(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Mp4 => "mp4",
            AudioFormat::Flac => "flac",
            AudioFormat::Mpeg => "mpeg",
            AudioFormat::Mpga => "mpga",
            AudioFormat::M4a => "m4a",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Wav => "wav",
            AudioFormat::Webm => "webm",
        }
    }
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "mp3" => Some(AudioFormat::Mp3),
            "mp4" => Some(AudioFormat::Mp4),
            "flac" => Some(AudioFormat::Flac),
            "mpeg" => Some(AudioFormat::Mpeg),
            "mpga" => Some(AudioFormat::Mpga),
            "m4a" => Some(AudioFormat::M4a),
            "ogg" => Some(AudioFormat::Ogg),
            "wav" => Some(AudioFormat::Wav),
            "webm" => Some(AudioFormat::Webm),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ResponseFormat {
    Json,
    Text,
    Srt,
    VerboseJson,
    Vtt,
}

#[derive(Debug, Default)]
pub struct TranscribeRequestBuilder {
    pub(crate) audio: Option<Vec<u8>>,
    pub(crate) model: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) prompt: Option<String>,
    pub(crate) format: Option<AudioFormat>,
    pub(crate) response_format: Option<ResponseFormat>,
    pub(crate) temperature: Option<f64>,
    pub(crate) openai: Option<OpenAi>,
}

#[derive(Debug, Error)]
pub enum TranscibeRequestBuilderError {
    #[error("File not set")]
    FileNotSet,
    #[error("Model not set")]
    ModelNotSet,
    #[error("Client not set")]
    ClientNotSet,
    #[error("Format not set")]
    FormatNotSet,
}

#[derive(Debug)]
pub enum Audio {
    Bytes(Vec<u8>),
    File(String),
}

impl From<Vec<u8>> for Audio {
    fn from(bytes: Vec<u8>) -> Self {
        Audio::Bytes(bytes)
    }
}

impl From<String> for Audio {
    fn from(file: String) -> Self {
        Audio::File(file)
    }
}

#[derive(Debug)]
pub struct TranscribeRequest {
    audio: Vec<u8>,
    model: String,
    language: Option<String>,
    prompt: Option<String>,
    format: AudioFormat,
    response_format: Option<ResponseFormat>,
    temperature: Option<f64>,
    openai: OpenAi,
}

impl TranscribeRequestBuilder {
    pub fn audio<T: Into<Audio>>(mut self, audio: T) -> Self {
        match audio.into() {
            Audio::Bytes(bytes) => self.audio = Some(bytes),
            Audio::File(file) => {
                let bytes = std::fs::read(&file).unwrap();
                let format = AudioFormat::from_extension(file.split('.').last().unwrap()).unwrap();
                self.format = Some(format);
                self.audio = Some(bytes);
            }
        }
        self
    }
    pub fn format(mut self, format: AudioFormat) -> Self {
        self.format = Some(format);
        self
    }
    pub fn model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }
    pub fn language(mut self, language: &str) -> Self {
        self.language = Some(language.to_string());
        self
    }
    pub fn prompt(mut self, prompt: &str) -> Self {
        self.prompt = Some(prompt.to_string());
        self
    }
    pub fn response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }
    pub fn openai(mut self, client: impl Into<OpenAi>) -> Self {
        self.openai = Some(client.into());
        self
    }
    pub fn build(self) -> Result<TranscribeRequest, TranscibeRequestBuilderError> {
        let Some(audio) = self.audio else {
            return Err(TranscibeRequestBuilderError::FileNotSet);
        };
        let Some(model) = self.model else {
            return Err(TranscibeRequestBuilderError::ModelNotSet);
        };
        let Some(format) = self.format else {
            return Err(TranscibeRequestBuilderError::FormatNotSet);
        };
        let Some(openai) = self.openai else {
            return Err(TranscibeRequestBuilderError::ClientNotSet);
        };
        Ok(TranscribeRequest {
            audio,
            model,
            language: self.language,
            prompt: self.prompt,
            format,
            response_format: self.response_format,
            temperature: self.temperature,
            openai,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct TranscribeJsonResponse {
    pub text: String,
}

impl TranscribeRequest {
    pub fn builder() -> TranscribeRequestBuilder {
        TranscribeRequestBuilder::default()
    }
    pub async fn send<O: DeserializeOwned>(&self) -> Result<O, ApiRequestError> {
        let url = format!("{}/{}", BASE_URL, API_URL);
        let file = multipart::Part::bytes(self.audio.to_owned())
            .file_name(format!("audio.{}", self.format.to_extension()))
            .mime_str(self.format.to_mime())?;
        let mut form = multipart::Form::new()
            .part("file", file)
            .text("model", self.model.clone());
        if let Some(language) = &self.language {
            form = form.text("language", language.to_owned());
        }
        if let Some(prompt) = &self.prompt {
            form = form.text("prompt", prompt.to_owned());
        }
        if let Some(response_format) = &self.response_format {
            form = form.text("response_format", format!("{:?}", response_format));
        }
        if let Some(temperature) = self.temperature {
            form = form.text("temperature", temperature.to_string());
        }
        let req = self
            .openai
            .client
            .post(&url)
            .bearer_auth(&self.openai.api_key)
            .multipart(form);
        let res = req.send().await?;
        if res.status().is_success() {
            let data: O = res.json().await?;
            Ok(data)
        } else {
            let error_response: ErrorResponse = res.json().await?;
            Err(ApiRequestError::InvalidRequestError {
                message: error_response.error.message,
                param: error_response.error.param,
                code: error_response.error.code,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OpenAiBuilder;

    #[tokio::test]
    async fn transcribe_test() {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let client = reqwest::Client::new();
        let openai = OpenAiBuilder::default()
            .api_key(api_key)
            .client(&client)
            .build()
            .unwrap();
        let audio = std::fs::read(
            "/home/ribelo/downloads/1 Comparison Of Vernacular And Refined Speech.mp3",
        )
        .unwrap();
        let res: TranscribeJsonResponse = TranscribeRequestBuilder::default()
            .audio(audio)
            .format(AudioFormat::Mp3)
            .model("whisper-1")
            .openai(openai)
            .build()
            .unwrap()
            .send()
            .await
            .unwrap();
        dbg!(res);
    }
}
