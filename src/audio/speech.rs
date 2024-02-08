use serde::Serialize;
use thiserror::Error;

use crate::{ApiRequestError, ErrorResponse, OpenAi, BASE_URL};

const MAX_INPUT_LENGTH: usize = 4096;
const MIN_SPEED: f32 = 0.25;
const MAX_SPEED: f32 = 4.0;
const API_URL: &str = "v1/audio/speech";

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    MP3,
    AAC,
    FLAC,
    OPUS,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct SpeechRequest {
    model: String,
    input: String,
    voice: String,
    response_format: ResponseFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
    #[serde(skip)]
    openai: OpenAi,
}

#[derive(Debug, Default)]
pub struct SpeechRequestBuilder {
    model: Option<String>,
    input: Option<String>,
    voice: Option<String>,
    response_format: Option<ResponseFormat>,
    speed: Option<f32>,
    openai: Option<OpenAi>,
}

#[derive(Debug, Error)]
pub enum SpeechRequestBuilderError {
    #[error("Input text is too long")]
    TextTooLong,
    #[error("Speed must be between {} and {}", MIN_SPEED, MAX_SPEED)]
    SpeedOutOfRange,
    #[error("Model not set")]
    ModelNotSet,
    #[error("Client not set")]
    ClientNotSet,
    #[error("Response format not set")]
    ResponseFormatNotSet,
    #[error("Input not set")]
    InputNotSet,
    #[error("Voice not set")]
    VoiceNotSet,
}

impl SpeechRequestBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn model(mut self, model: impl AsRef<str>) -> Self {
        self.model = Some(model.as_ref().to_owned());
        self
    }
    pub fn input(mut self, input: impl AsRef<str>) -> Self {
        self.input = Some(input.as_ref().to_owned());
        self
    }
    pub fn voice(mut self, voice: impl AsRef<str>) -> Self {
        self.voice = Some(voice.as_ref().to_owned());
        self
    }
    pub fn response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }
    pub fn client(mut self, client: OpenAi) -> Self {
        self.openai = Some(client);
        self
    }
    pub fn build(self) -> Result<SpeechRequest, SpeechRequestBuilderError> {
        if self.input.as_ref().unwrap().len() > MAX_INPUT_LENGTH {
            return Err(SpeechRequestBuilderError::TextTooLong);
        }
        if let Some(speed) = self.speed {
            if !(MIN_SPEED..=MAX_SPEED).contains(&speed) {
                return Err(SpeechRequestBuilderError::SpeedOutOfRange);
            }
        }
        let Some(model) = self.model else {
            return Err(SpeechRequestBuilderError::ModelNotSet);
        };
        let Some(input) = self.input else {
            return Err(SpeechRequestBuilderError::InputNotSet);
        };
        let Some(voice) = self.voice else {
            return Err(SpeechRequestBuilderError::VoiceNotSet);
        };
        let Some(response_format) = self.response_format else {
            return Err(SpeechRequestBuilderError::ResponseFormatNotSet);
        };
        let Some(openai) = self.openai else {
            return Err(SpeechRequestBuilderError::ClientNotSet);
        };
        Ok(SpeechRequest {
            model,
            input,
            voice,
            response_format,
            speed: self.speed,
            openai,
        })
    }
}

impl TryFrom<SpeechRequestBuilder> for SpeechRequest {
    type Error = SpeechRequestBuilderError;
    fn try_from(builder: SpeechRequestBuilder) -> Result<Self, Self::Error> {
        builder.build()
    }
}

impl SpeechRequest {
    pub async fn send(&self) -> Result<Vec<u8>, ApiRequestError> {
        let url = format!("{}/{}", BASE_URL, API_URL);
        let request = self
            .openai
            .client
            .post(&url)
            .bearer_auth(&self.openai.api_key)
            .json(self);
        let response = request.send().await?;
        if response.status().is_success() {
            Ok(response.bytes().await?.to_vec())
        } else {
            let error_response: ErrorResponse = response.json().await?;
            Err(ApiRequestError::InvalidRequestError {
                message: error_response.error.message,
                param: error_response.error.param,
                code: error_response.error.code,
            })
        }
    }
}

impl OpenAi {
    pub fn speech(&self) -> SpeechRequestBuilder {
        SpeechRequestBuilder {
            openai: Some(self.clone()),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{audio::speech::ResponseFormat::MP3, OpenAiBuilder};

    #[tokio::test]
    async fn speech_test() {
        let input = r#"
Najszlachetniejsze zwierzęta odmawiają rozmnażania się w niewoli. Wiele zwierząt, nie tylko człowiek, wybiera śmierć, gdy są uwięzione.Ale jeśli to nie wystarczy, to musimy zrozumieć zwierzęta w inny sposób. Kiedy myśliciele mówią o "psychologii ewolucyjnej", często abstrahują od drożdży do zwierząt i ludzi, ale to jest cofanie się. W świecie naukowców, jak wszędzie indziej, istnieje swoista socjologia, co prowadzi do wielu pomyłek na temat biologii i idei ewolucji. Myślisz, że dostajesz obiektywną prawdę, ale umysły biologów są ogólnie bardzo ograniczone. Prawda jest taka, że największe umysły zawsze wybierały fizykę spośród nauk, a może potem chemię. Dopiero niedawno, ale nawet teraz, biologia daje mało możliwości na rodzaj myślenia, który penetruje tajemnicę natury, na rodzaj wglądu w fizyczne relacje, który przyciąga najlepsze umysły naukowe. Historia ich na ogół przedstawia jako grupę wykazującą umiarkowane zdolności. Schopenhauer z pogardą odnosił się do tych, którzy mają swoje "katalogi małp" i myślą, że rozumieją naturę. Darwin sam, Nietzsche nazwał go małym umysłem, takim rachmistrzem, który lubi zbierać wiele małych faktów i syntetyzować z tego niezdarną teorię. Teoria jest niezdarna i pełna dziur. To jest główny powód, dla którego kreacjoniści, którzy również są w błędzie, byli w stanie go podważyć, podczas gdy nigdy nie byli w stanie podważyć teoretycznej fizyki. Jest wiele nieuczciwości i głupoty wśród naukowców i biologów, kiedy mówią o ewolucji i życiu.
            "#;
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let client = reqwest::Client::new();
        let openai = OpenAiBuilder::default()
            .api_key(api_key)
            .client(&client)
            .build()
            .unwrap();
        let mp3 = openai
            .speech()
            .model("tts-1-hd")
            .input(input)
            .voice("onyx")
            .response_format(MP3)
            .speed(1.2)
            .build()
            .unwrap()
            .send()
            .await;
        std::fs::write("test.mp3", mp3.unwrap()).unwrap();
    }
}
