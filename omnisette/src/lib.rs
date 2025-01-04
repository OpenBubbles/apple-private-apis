//! A library to generate "anisette" data. Docs are coming soon.
//!
//! If you want an async API, enable the `async` feature.
//!
//! If you want remote anisette, make sure the `remote-anisette` feature is enabled. (it's currently on by default)

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
#[cfg(feature = "remote-clearadi")]
use anisette_clearadi::ClearADIClient;
#[cfg(target_os = "macos")]
use aos_kit::AOSKitAnisetteProvider;
use thiserror::Error;
use tokio::sync::Mutex;

#[cfg(feature = "remote-clearadi")]
pub mod anisette_clearadi;

#[cfg(feature = "remote-anisette-v3")]
pub mod remote_anisette_v3;

#[cfg(target_os = "macos")]
pub mod aos_kit;

#[allow(dead_code)]
pub struct AnisetteHeaders;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AnisetteError {
    #[allow(dead_code)]
    #[error("Unsupported device")]
    UnsupportedDevice,
    #[error("Invalid argument {0}")]
    InvalidArgument(String),
    #[error("Anisette not provisioned!")]
    AnisetteNotProvisioned,
    #[error("Plist serialization error {0}")]
    PlistError(#[from] plist::Error),
    #[error("Request Error {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[cfg(feature = "remote-anisette-v3")]
    #[error("Provisioning socket error {0}")]
    WsError(#[from] tokio_tungstenite::tungstenite::error::Error),
    #[cfg(feature = "remote-anisette-v3")]
    #[error("JSON error {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("IO error {0}")]
    IOError(#[from] io::Error),
    #[error("Invalid library format")]
    InvalidLibraryFormat,
    #[error("Misc")]
    Misc,
    #[error("Missing Libraries")]
    MissingLibraries,
    #[cfg(feature = "remote-clearadi")]
    #[error("ClearADI Error {0}")]
    ClearADIError(#[from] clearadi::ClearAdiError),
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error)
}

pub const DEFAULT_ANISETTE_URL: &str = "https://ani.f1sh.me/";

pub const DEFAULT_ANISETTE_URL_V3: &str = "https://ani.sidestore.io";

pub trait AnisetteProvider {
    fn get_anisette_headers(&mut self) -> impl std::future::Future<Output = Result<HashMap<String, String>, AnisetteError>> + Send;
}

// conditionally compile this
#[cfg(not(target_os = "macos"))]
pub type DefaultAnisetteProvider = ClearADIClient;
#[cfg(not(target_os = "macos"))]
pub fn default_provider(info: LoginClientInfo, path: PathBuf) -> ArcAnisetteClient<DefaultAnisetteProvider> {
    Arc::new(Mutex::new(AnisetteClient::new(ClearADIClient {
        login_info: info,
        configuration_path: path
    })))
}


#[cfg(target_os = "macos")]
pub type DefaultAnisetteProvider = AOSKitAnisetteProvider<'static>;
#[cfg(target_os = "macos")]
pub fn default_provider(info: LoginClientInfo, path: PathBuf) -> ArcAnisetteClient<DefaultAnisetteProvider> {
    Arc::new(Mutex::new(AnisetteClient::new(AOSKitAnisetteProvider::new().expect("Failed to load anisette provider?"))))
}

pub type ArcAnisetteClient<T> = Arc<Mutex<AnisetteClient<T>>>;


pub struct AnisetteClient<T: AnisetteProvider> {
    provider: T,
    cached_headers: HashMap<String, String>,
    generated_at: SystemTime,
}

impl<T: AnisetteProvider> AnisetteClient<T> {
    pub fn new(p: T) -> AnisetteClient<T> {
        AnisetteClient {
            provider: p,
            cached_headers: HashMap::new(),
            generated_at: SystemTime::UNIX_EPOCH,
        }
    }

    pub async fn get_headers(&mut self) -> Result<&HashMap<String, String>, AnisetteError> {
        let last_generated = SystemTime::now().duration_since(self.generated_at).unwrap_or(Duration::from_secs(120));

        if last_generated > Duration::from_secs(60) {
            self.cached_headers = self.provider.get_anisette_headers().await?;
            self.generated_at = SystemTime::now();
        }

        Ok(&self.cached_headers)
    }
}

#[derive(Clone, Debug, Default)]
pub struct LoginClientInfo {
    pub ak_context_type: String,
    pub client_app_name: String,
    pub client_bundle_id: String,
    pub mme_client_info: String,
    pub mme_client_info_akd: String,
    pub akd_user_agent: String,
    pub browser_user_agent: String,
    pub hardware_headers: HashMap<String, String>,
    pub push_token: Option<String>,
    pub update_account_bundle_id: String,
}


