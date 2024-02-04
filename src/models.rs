use crate::{ApiRequestError, OpenAi};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    id: String,
    object: String,
    owned_by: String,
    permission: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelList {
    data: Vec<Model>,
    object: String,
}

impl From<Model> for String {
    fn from(value: Model) -> Self {
        value.id
    }
}

impl OpenAi {
    pub async fn get_models(&self) -> Result<ModelList, ApiRequestError> {
        let url = "https://api.openai.com/v1/models";
        let response = self
            .client
            .get(url)
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .json::<ModelList>()
            .await?;
        Ok(response)
    }

    pub async fn get_model(&self, model_id: &str) -> Result<Model, ApiRequestError> {
        let url = format!("https://api.openai.com/v1/models/{}", model_id);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json::<Model>()
            .await?;
        Ok(response)
    }
}
