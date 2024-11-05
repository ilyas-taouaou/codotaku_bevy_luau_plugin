use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct ScriptAsset {
    code: String,
}

impl ScriptAsset {
    fn new(code: String) -> Self {
        ScriptAsset { code }
    }
    pub fn code(&self) -> &str {
        &self.code
    }
}

#[derive(Default)]
pub struct ScriptAssetLoader {}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ScriptAssetLoaderError {
    #[error("Could not load script asset: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for ScriptAssetLoader {
    type Asset = ScriptAsset;

    type Settings = ();

    type Error = ScriptAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut code = String::new();
        reader.read_to_string(&mut code).await?;
        Ok(ScriptAsset::new(code))
    }

    fn extensions(&self) -> &[&str] {
        &["luau"]
    }
}
