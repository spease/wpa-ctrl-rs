use err_derive::Error;
use std::collections::HashMap;
use core::num::ParseIntError;
use core::str::ParseBoolError;
fn fetchkey(map: &HashMap<String, String>, key: &'static str) -> Result<String, HostAPDError> {
    match map.get(key) {
        Some(val) => Ok(val.clone()),
        None => Err(HostAPDError::MissingKey(key)),
    }
}

#[derive(Debug, Error)]
pub enum HostAPDError {
    #[error(display = "failed to find key: {}", _0)]
    MissingKey(&'static str),
    #[error(display = "{}", _0)]
    ParseIntError(#[error(source)] ParseIntError),
    #[error(display = "{}", _0)]
    ParseBoolError(#[error(source)] ParseBoolError),
}

pub trait MIBVariables 
where Self: Sized {
    type Error: std::error::Error;
    fn from_mib_vars(vals: &HashMap<String, String>) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BackendAuth {
    pub State: u8,
    pub Fails: u8,
    pub Successes: u8,
}

/// port access entity
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PAE {
    pub port_number: i32,
    pub port_protocol_version: i32,
    pub port_capabilities: i32,
    pub port_initialize: i32,
    pub port_reauthenticate: bool,
}

impl MIBVariables for PAE {
    type Error = HostAPDError;
    fn from_mib_vars(vals: &HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Self {
            port_number: fetchkey(&vals, "dot1xPaePortNumber")?.parse()?,
            port_protocol_version: fetchkey(&vals, "dot1xPaePortProtocolVersion")?.parse()?,
            port_capabilities: fetchkey(&vals, "dot1xPaePortCapabilities")?.parse()?,
            port_initialize: fetchkey(&vals, "dot1xPaePortInitialize")?.parse()?,
            port_reauthenticate: fetchkey(&vals, "dot1xPaePortReauthenticate")?.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Session {
    /// hostapd session idenfier use to identify a connection to a device connected to the wifi network
    pub id: String,
    
    pub auth_method: i32,
    /// how long the session has been running for
    pub time: i64,
    pub termination_cause: i32,
    /// the identity / username used in the hostapd session, that the user enters when logging in
    pub username: String,
}

impl MIBVariables for Session {
    type Error = HostAPDError;
    fn from_mib_vars(vals: &HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: fetchkey(&vals, "dot1xAuthSessionId")?,
            username: fetchkey(&vals, "dot1xAuthSessionUserName")?,
            auth_method: fetchkey(&vals, "dot1xAuthSessionAuthenticMethod")?.parse()?,
            time: fetchkey(&vals, "dot1xAuthSessionTime")?.parse()?,
            termination_cause: fetchkey(&vals, "dot1xAuthSessionTerminateCause")?.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Auth {
    pub ControlledPortStatus: u8,
    pub ControlledPortControl: u8,
    pub SuccessesWhileAuthenticating: u8,
    pub TimeoutsWhileAuthenticating: u8,
    pub FailWhileAuthenticating: u8,
    pub EapStartsWhileAuthenticating: u8,
    pub EapLogoffWhileAuthenticating: u8,
    pub ReauthsWhileAuthenticated: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Eapol {
    pub frames_rx: i32,
    pub frames_tx: i32,
    pub start_frames_rx: i32,
    pub logoff_frames_rx: i32,
    pub resp_id_frames_rx: i32,
    pub resp_frames_rx: i32,
    pub req_id_frames_tx: i32,
    pub req_frames_tx: i32,
}

impl MIBVariables for Eapol {
    type Error = HostAPDError;
    fn from_mib_vars(vals: &HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Self {
            frames_rx: fetchkey(&vals, "dot1xAuthEapolFramesRx")?.parse()?,
            frames_tx: fetchkey(&vals, "dot1xAuthEapolFramesTx")?.parse()?,
            start_frames_rx: fetchkey(&vals, "dot1xAuthEapolStartFramesRx")?.parse()?,
            logoff_frames_rx: fetchkey(&vals, "dot1xAuthEapolLogoffFramesRx")?.parse()?,
            resp_id_frames_rx: fetchkey(&vals, "dot1xAuthEapolRespIdFramesRx")?.parse()?,
            resp_frames_rx: fetchkey(&vals, "dot1xAuthEapolRespFramesRx")?.parse()?,
            req_id_frames_tx: fetchkey(&vals, "dot1xAuthEapolReqIdFramesTx")?.parse()?,
            req_frames_tx: fetchkey(&vals, "dot1xAuthEapolReqFramesTx")?.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Backend {
    pub Responses: u32,
    pub BackendAccessChallenges: u32,
    pub BackendOtherRequestsToSupplicant: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct dot1xAuth {
    pub PaeState: u8,
    pub AdminControlledDirections: u8,
    pub OperControlledDirections: u8,
    pub QuietPeriod: u32,
    pub ServerTimeout: u32,
    pub ReAuthPeriod: u32,
    pub ReAuthEnabled: bool,
    pub KeyTxEnabled: bool,
    pub InvalidEapolFramesRx: u32,
    pub EapLengthErrorFramesRx: u32,
    pub LastEapolFrameVersion: u32,
    /// mac address
    pub LastEapolFrameSource: String,
    pub EntersConnecting: u8,
    pub EapLogoffsWhileConnecting: u8,
    pub EntersAuthenticating: u8,
    pub backend: Backend,
    pub eapol: Eapol,
    pub auth: Auth,
    pub backend_auth: BackendAuth,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Flag {
    Auth,
    ASSOC,
    AUTHORIZED,
    WMM,
    HT,
    VHT,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PacketRecord {
    pub rx_packets: i64,
    pub tx_packets: i64,
    pub rx_bytes: i64,
    pub tx_bytes: i64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Station {
    pub flags: Vec<Flag>,
    pub aid: i32,
    pub capability: i32,
    pub listen_interval: u16,
    pub supported_rates: Vec<u8>,
    pub timeout_next: String,
    pub wpa: u8,
    pub AKMSuiteSelector: String,
    pub last_eap_type_as: u8,
    pub last_eap_type_sta: u8,
    pub inactive_msec: i32,
    pub signal: i32,
    pub rx_rate_info: i32,
    pub tx_rate_info: String,
    pub rx_vht_mcs_map: i32,
    pub tx_vht_mcs_map: i32,
    pub ht_mcs_bitmask: i64,
    pub connected_time: i64,
    pub supp_op_classes: String,
    pub min_txpower: u8,
    pub max_txpower: u8,
    pub vht_caps_info: i32,
    pub ht_caps_info: i32,
    pub ext_capab: i64,
    pub dot1xpae: PAE,
    pub dot1xauth: dot1xAuth,
    pub dot11RSNAStats: dot11RSNAStats,
}

impl Station {
    /*/// this method expects values in the form key=value
    pub fn from_string(value: String) {
        let keyval = Self::key_value(value);
        Self::from_key_values(keyval)
    }

    pub fn from_key_values(keyval: HashMap<String, String>) -> Self {
        let mut selfmap = HashMap::new();
        let mut dot1xmap = HashMap::new();
        let mut dot11rnamap = HashMap::new();
        for (key, value) in keyval.into_iter() {
            if key.contains("dot1x") {
                let newkey = key.split_at(5).1.to_string();
                dot1xmap.insert(newkey, value);
            }else if key.contains("dot11RSNAStats") {
                let newkey = key.split_at(14).1.to_string();
                dot11rnamap.insert(key, value);
            }else{
                selfmap.insert(key, value);
            }
        }
        let dot1x = dot1x::from_key_values(dot1xmap);
        let dot11RSNAStats = dot11RSNAStats::from_key_values(dot11rnamap);
        Self::from_values_internal()
    }
    fn from_values_internal(dot1x: dot1x, dot11RSNAStats: dot11RSNAStats, selfmap: HashMap<String, String>) {

    }*/
    pub fn key_value(value: &String) -> HashMap<String, String> {
        let mut retvals = HashMap::new();
        for (key, value_opt) in value.split('\n').map(|line| { 
            let mut parts = line.split('=');
            (parts.next().unwrap(), parts.next())
        }) {
            if let Some(value) = value_opt {
                retvals.insert(key.to_string(), value.to_string());
            }
        }
        retvals
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct dot11RSNAStats {
    /// mac address
    pub sta_addr: String,
    pub version: i32,
    pub selected_pairwise_cipher: String,
    pub tkip_local_mic_failures: i32,
    pub tkip_remote_mic_failures: i32,
}

impl MIBVariables for dot11RSNAStats {
    type Error = HostAPDError;
    fn from_mib_vars(vals: &HashMap<String, String>) -> Result<Self, Self::Error> {
        Ok(Self{
            sta_addr: fetchkey(&vals, "dot11RSNAStatsSTAAddress")?,
            version: fetchkey(&vals, "dot11RSNAStatsVersion")?.parse()?,
            selected_pairwise_cipher: fetchkey(&vals, "dot11RSNAStatsSelectedPairwiseCipher")?,
            tkip_local_mic_failures: fetchkey(&vals, "dot11RSNAStatsTKIPLocalMICFailures")?.parse()?,
            tkip_remote_mic_failures: fetchkey(&vals, "dot11RSNAStatsTKIPRemoteMICFailures")?.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct hostapdWPAPTK {
    pub State: u8,
    pub GroupState: u8,
}

pub struct HostAPD {
    ctrl: crate::Client,
}

impl HostAPD {
    pub fn new(ctrl: crate::Client) -> Self {
        Self {
            ctrl
        }
    }
    #[cfg(feature = "async")]
    pub async fn get_stations(&mut self) -> Result<Option<Vec<HashMap<String, String>>>, Box<dyn std::error::Error>> {
        let mut stations = Vec::new();
        let mut station = self.ctrl.request("STA-FIRST").await?;
        if station != "" {
            stations.push(Station::key_value(&station));
        }else{
            return Ok(None);
        }
        let mut addr: String = station.split('\n').next().unwrap().to_string();
    
        while station != "" && station != "UNKNOWN COMMAND\n" {
            
            station = self.ctrl.request(format!("STA-NEXT {}", addr).as_str()).await?;
            if station != "" {
                addr = station.split('\n').next().unwrap().to_string();
                stations.push(Station::key_value(&station));
            }
        }
        Ok(Some(stations))
    }

    #[cfg(not(feature = "async"))]
    pub fn get_stations(&mut self) -> Result<Option<Vec<HashMap<String, String>>>, Box<dyn std::error::Error>> {
        
        let mut stations = Vec::new();
        let mut station = self.ctrl.request("STA-FIRST")?;
        
        if station != "" {
            stations.push(Station::key_value(&station));
        }else{
            return Ok(None);
        }
        let mut addr: String = station.split('\n').next().unwrap().to_string();
    
        while station != "" && station != "UNKNOWN COMMAND\n" {
            station = self.ctrl.request(format!("STA-NEXT {}", addr).as_str())?;
            addr = station.split('\n').next().unwrap().to_string();
            stations.push(Station::key_value(&station));
        }
        Ok(Some(stations))
    }
}