//! A library to generate "anisette" data. Docs are coming soon.
//!
//! If you want an async API, enable the `async` feature.
//!
//! If you want remote anisette, make sure the `remote-anisette` feature is enabled. (it's currently on by default)

use crate::adi_proxy::{ADIProxyAnisetteProvider, ConfigurableADIProxy};
use crate::anisette_headers_provider::AnisetteHeadersProvider;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use adi_proxy::ADIError;
use thiserror::Error;

pub mod adi_proxy;
pub mod anisette_headers_provider;
pub mod store_services_core;

pub mod anisette_clearadi;

#[cfg(feature = "remote-anisette-v3")]
pub mod remote_anisette_v3;

// #[cfg(target_os = "macos")]
// pub mod aos_kit;

#[cfg(feature = "remote-anisette")]
pub mod remote_anisette;

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
    #[error("ADI error {0}")]
    ADIError(#[from] ADIError),
    #[error("Invalid library format")]
    InvalidLibraryFormat,
    #[error("Misc")]
    Misc,
    #[error("Missing Libraries")]
    MissingLibraries,
    #[error("ClearADI Error {0}")]
    ClearADIError(#[from] clearadi::ClearAdiError),
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error)
}

pub const DEFAULT_ANISETTE_URL: &str = "https://ani.f1sh.me/";

pub const DEFAULT_ANISETTE_URL_V3: &str = "https://ani.sidestore.io";

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
}

#[derive(Clone, Debug)]
pub struct AnisetteConfiguration {
    anisette_url: String,
    anisette_url_v3: String,
    configuration_path: PathBuf,
    pub client_info: LoginClientInfo,
}

impl Default for AnisetteConfiguration {
    fn default() -> Self {
        AnisetteConfiguration::new()
    }
}

impl AnisetteConfiguration {
    pub fn new() -> AnisetteConfiguration {
        AnisetteConfiguration {
            anisette_url: DEFAULT_ANISETTE_URL.to_string(),
            anisette_url_v3: DEFAULT_ANISETTE_URL_V3.to_string(),
            configuration_path: PathBuf::new(),
            client_info: LoginClientInfo {
                ak_context_type: "imessage".to_string(),
                client_app_name: "Messages".to_string(),
                client_bundle_id: "com.apple.MobileSMS".to_string(),
                mme_client_info_akd: "<MacBookPro18,3> <macOS;13.2.1;22D68> <com.apple.AuthKit/1 (com.apple.akd/1.0)>".to_string(),
                mme_client_info: "<MacBookPro18,3> <macOS;13.2.1;22D68> <com.apple.AuthKit/1 (com.apple.akd/1.0)>".to_string(),
                akd_user_agent: "akd/1.0 CFNetwork/1494.0.7 Darwin/23.4.0".to_string(),
                browser_user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)".to_string(),
                hardware_headers: HashMap::new(),
                push_token: None,
            },
        }
    }

    pub fn anisette_url(&self) -> &String {
        &self.anisette_url
    }

    pub fn configuration_path(&self) -> &PathBuf {
        &self.configuration_path
    }

    pub fn set_anisette_url(mut self, anisette_url: String) -> AnisetteConfiguration {
        self.anisette_url = anisette_url;
        self
    }

    pub fn set_configuration_path(mut self, configuration_path: PathBuf) -> AnisetteConfiguration {
        self.configuration_path = configuration_path;
        self
    }

    pub fn set_client_info(mut self, client_info: LoginClientInfo) -> AnisetteConfiguration {
        self.client_info = client_info;
        self
    }
}

pub enum AnisetteHeadersProviderType {
    Local,
    Remote,
}

pub struct AnisetteHeadersProviderRes {
    pub provider: Box<dyn AnisetteHeadersProvider>,
    pub provider_type: AnisetteHeadersProviderType,
}

impl AnisetteHeadersProviderRes {
    pub fn local(provider: Box<dyn AnisetteHeadersProvider>) -> AnisetteHeadersProviderRes {
        AnisetteHeadersProviderRes {
            provider,
            provider_type: AnisetteHeadersProviderType::Local,
        }
    }

    pub fn remote(provider: Box<dyn AnisetteHeadersProvider>) -> AnisetteHeadersProviderRes {
        AnisetteHeadersProviderRes {
            provider,
            provider_type: AnisetteHeadersProviderType::Remote,
        }
    }
}

impl AnisetteHeaders {
    pub fn get_anisette_headers_provider(
        configuration: AnisetteConfiguration,
    ) -> Result<AnisetteHeadersProviderRes, AnisetteError> {
        // #[cfg(target_os = "macos")]
        // if let Ok(prov) = aos_kit::AOSKitAnisetteProvider::new() {
        //     return Ok(AnisetteHeadersProviderRes::local(Box::new(prov)));
        // }

        // TODO: handle Err because it will just go to remote anisette and not tell the user anything
        if let Ok(ssc_anisette_headers_provider) =
            AnisetteHeaders::get_ssc_anisette_headers_provider(configuration.clone())
        {
            return Ok(ssc_anisette_headers_provider);
        }

        return Ok(AnisetteHeadersProviderRes::remote(Box::new(
            anisette_clearadi::ClearAdiProvider::new(configuration.configuration_path.clone(), configuration.client_info.clone()),
        )));

        #[cfg(feature = "remote-anisette-v3")]
        return Ok(AnisetteHeadersProviderRes::remote(Box::new(
            remote_anisette_v3::RemoteAnisetteProviderV3::new(configuration.anisette_url_v3, configuration.configuration_path.clone(), configuration.client_info.clone()),
        )));

        #[cfg(feature = "remote-anisette")]
        return Ok(AnisetteHeadersProviderRes::remote(Box::new(
            remote_anisette::RemoteAnisetteProvider::new(configuration.anisette_url),
        )));

        #[cfg(not(feature = "remote-anisette"))]
        bail!(AnisetteMetaError::UnsupportedDevice)
    }

    pub fn get_ssc_anisette_headers_provider(
        configuration: AnisetteConfiguration,
    ) -> Result<AnisetteHeadersProviderRes, AnisetteError> {
        let mut ssc_adi_proxy = store_services_core::StoreServicesCoreADIProxy::new(
            configuration.configuration_path(),
        )?;
        let config_path = configuration.configuration_path();
        ssc_adi_proxy.set_provisioning_path(config_path.to_str().ok_or(
            AnisetteError::InvalidArgument("configuration.configuration_path".to_string()),
        )?)?;
        Ok(AnisetteHeadersProviderRes::local(Box::new(
            ADIProxyAnisetteProvider::new(ssc_adi_proxy, config_path.to_path_buf())?,
        )))
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

    pub fn init_logger() {
        if TermLogger::init(
            LevelFilter::Trace,
            ConfigBuilder::new()
                .set_target_level(LevelFilter::Error)
                .add_filter_allow_str("omnisette")
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )
        .is_ok()
        {}
    }
}
