use std::{error::Error, ffi, fmt::Display, ptr::null_mut, slice};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ClearAdiError(i32);

impl Display for ClearAdiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct ProvisionedMachine {
    pub client_secret: [u8; 32],
    pub mid: [u8; 60],
    pub metadata: Vec<u8>,
}

impl ProvisionedMachine {
    pub fn generate_otp(&self) -> Vec<u8> {
        todo!()
    }

    pub fn gen_2fa_code(&self) -> u32 {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProvisionedMachineSerde {
    pub client_secret: [u8; 32],
    pub mid: Vec<u8>,
    pub metadata: Vec<u8>,
}

impl Error for ClearAdiError { }


pub struct ProvisioningSession(*mut ffi::c_void);
unsafe impl Send for ProvisioningSession { }

impl ProvisioningSession {
    pub fn new(spim: &[u8], _hostuuid: &[u8], _dsid: i64) -> Result<(Self, Vec<u8>), ClearAdiError> {
        todo!()
    }

    pub fn finish(self, tk: &[u8], ptm: &[u8]) -> Result<ProvisionedMachine, ClearAdiError> {
        todo!()
    }
}