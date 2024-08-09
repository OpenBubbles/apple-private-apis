use crate::Error;
use omnisette::{AnisetteConfiguration, AnisetteHeaders};
use plist::{Dictionary, Value};
use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, Clone)]
pub struct AnisetteData {
    pub base_headers: HashMap<String, String>,
    pub generated_at: SystemTime,
    pub config: AnisetteConfiguration,
}

impl AnisetteData {
    /// Fetches the data at an anisette server
    pub async fn new(config: AnisetteConfiguration) -> Result<Self, crate::Error> {
        let mut b = AnisetteHeaders::get_anisette_headers_provider(config.clone())?;
        let mut base_headers = b.provider.get_authentication_headers().await?;

        println!("anisette {base_headers:?}");

        Ok(AnisetteData { base_headers, generated_at: SystemTime::now(), config })
    }

    pub fn needs_refresh(&self) -> bool {
        let elapsed = self.generated_at.elapsed().unwrap();
        elapsed.as_secs() > 60
    }

    pub fn is_valid(&self) -> bool {
        let elapsed = self.generated_at.elapsed().unwrap();
        elapsed.as_secs() < 90
    }

    pub async fn refresh(&self) -> Result<Self, crate::Error> {
        Self::new(self.config.clone()).await
    }

    pub fn get_gsservice_headers(&self) -> HashMap<String, String> {
        // user must supply, content-type and accept
        // unaccounted headers: Accept-Encoding, Connection, Host
        const ACCEPTABLE_HEADERS: &[&'static str] = &["X-Apple-I-MD-LU", "X-Apple-I-MD-RINFO", "X-Apple-I-MD-M", "X-Apple-I-MD", "X-Mme-Device-Id"];
        self.base_headers.clone().into_iter().filter(|(key, _)| ACCEPTABLE_HEADERS.contains(&key.as_str()))
            .chain([
                ("X-Apple-AK-Context-Type", self.config.client_info.ak_context_type.as_str()),
                ("X-Apple-Client-App-Name", &self.config.client_info.client_app_name),
                ("X-Apple-I-Client-Bundle-Id", &self.config.client_info.client_bundle_id),
                ("X-MMe-Client-Info", &self.config.client_info.mme_client_info_akd),
                ("Accept-Language", "en-US,en;q=0.9"),
                ("User-Agent", &self.config.client_info.akd_user_agent),
            ].into_iter().map(|(a, b)| (a.to_string(), b.to_string())))
        .collect()
    }

    pub fn get_cpd_data(&self, request: &str) -> Dictionary {
        const ACCEPTABLE_HEADERS: &[&'static str] = &[
            "X-Apple-I-Client-Time",
            "X-Apple-I-MD",
            "X-Apple-I-MD-LU",
            "X-Apple-I-MD-M",
            "X-Apple-I-MD-RINFO",
            "X-Mme-Device-Id",
        ];
        self.base_headers.clone().into_iter().filter(|(key, _)| ACCEPTABLE_HEADERS.contains(&key.as_str()))
            .map(|(a, b)| (a, Value::String(b)))
            .chain(self.config.client_info.push_token.as_ref().map(|v| ("pktn".to_string(), Value::String(v.clone()))).into_iter())
            .chain([
                ("X-Apple-I-Device-Configuration-Mode", "0"),
                ("X-Apple-I-Request-UUID", request),
                ("X-Apple-Requested-Partition", "0"),
                ("X-Apple-Security-Upgrade-Context", "com.apple.authkit.generic"),
                ("capp", &self.config.client_info.client_app_name),
                ("cbid", &self.config.client_info.client_bundle_id),
                ("cou", "US"),
                ("loc", "en_US"),
                ("svct", &self.config.client_info.ak_context_type),
                
            ].into_iter().map(|(a, b)| (a.to_string(), Value::String(b.to_string()))))
            .chain([
                ("X-Apple-Offer-Security-Upgrade", Value::Boolean(true)),
                ("at", Value::Integer(0.into())),
                ("bootstrap", Value::Boolean(true)),
                ("ckgen", Value::Boolean(true)),
                ("fcd", Value::Boolean(true)),
                ("icdrsDisabled", Value::Boolean(false)),
                ("icscrec", Value::Boolean(true)),
                ("pbe", Value::Boolean(false)),
                ("prkgen", Value::Boolean(true)),
                ("webAccessEnabled", Value::Boolean(false)),
            ].into_iter().map(|(a, b)| (a.to_string(), b))
        ).chain(self.config.client_info.hardware_headers.clone().into_iter().map(|(a, b)| (a, Value::String(b)))).collect()
    }

    pub fn get_extra_headers(&self) -> HashMap<String, String> {
        // unaccounted headers: Accept-Encoding, Connection, Host, content-type
        const ACCEPTABLE_HEADERS: &[&'static str] = &["X-Apple-I-MD-LU", "X-Apple-I-MD-RINFO", "X-Apple-I-MD-M", "X-Apple-I-MD", "X-Mme-Device-Id"];
        self.base_headers.clone().into_iter().filter(|(key, _)| ACCEPTABLE_HEADERS.contains(&key.as_str()))
            .chain([
                ("X-Apple-Client-App-Name", self.config.client_info.client_app_name.as_str()),
                ("X-Apple-I-Client-Bundle-Id", &self.config.client_info.client_bundle_id),
                ("X-MMe-Client-Info", &self.config.client_info.mme_client_info),
                ("X-Apple-I-CDP-Circle-Status", "false"),
                ("X-Apple-I-ICSCREC", "true"),
                ("User-Agent", &self.config.client_info.browser_user_agent),
                ("Sec-Fetch-Site", "same-origin"), // diff
                ("X-Apple-Requested-Partition", "0"),
                ("X-Apple-I-DeviceUserMode", "0"),
                ("X-Apple-I-Locale", "en_US"),
                ("X-Apple-Security-Upgrade-Context", "com.apple.authkit.generic"),
                ("Accept-Language", "en-US,en;q=0.9"),
                ("X-Apple-I-PRK-Gen", "true"),
                ("Sec-Fetch-Mode", "cors"), // diff
                ("X-Apple-I-TimeZone", "UTC"),
                ("X-Apple-I-OT-Status", "false"),
                ("X-Apple-I-TimeZone-Offset", "0"), // check, -21600 denver
                ("X-MMe-Country", "US"),
                ("X-Apple-I-CDP-Status", "false"),
                ("X-Apple-I-Device-Configuration-Mode", "0"),
                ("Sec-Fetch-Dest", "empty"), // diff
                ("X-Apple-AK-Context-Type", self.config.client_info.ak_context_type.as_str()),
                ("X-Apple-I-CFU-State", "PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz4KPCFET0NUWVBFIHBsaXN0IFBVQkxJQyAiLS8vQXBwbGUvL0RURCBQTElTVCAxLjAvL0VOIiAiaHR0cDovL3d3dy5hcHBsZS5jb20vRFREcy9Qcm9wZXJ0eUxpc3QtMS4wLmR0ZCI+CjxwbGlzdCB2ZXJzaW9uPSIxLjAiPgo8YXJyYXkvPgo8L3BsaXN0Pgo="),
            ].into_iter().map(|(a, b)| (a.to_string(), b.to_string())))
            .chain(self.config.client_info.hardware_headers.clone())
        .collect()
    }
}
