use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connectivity {
    Unknown,
    None,
    Portal,
    Limited,
    Full,
}

#[cfg(feature = "display_hat")]
pub async fn check_connectivity() -> Result<Connectivity, Box<dyn std::error::Error>> {
    Ok(Connectivity::Full)
}

#[cfg(not(feature = "display_hat"))]
pub async fn check_connectivity() -> Result<Connectivity, Box<dyn std::error::Error>> {
    Ok(Connectivity::Full)
}

#[cfg(feature = "display_hat")]
pub fn scan_ssids(iface: &str) -> io::Result<Vec<String>> {
    // nmcli dev wifi list
    Ok(vec![])
}

#[cfg(not(feature = "display_hat"))]
pub fn scan_ssids(_iface: &str) -> io::Result<Vec<String>> {
    Ok(vec![])
}

#[cfg(feature = "display_hat")]
pub async fn add_connection(ssid: &str, password: &str) -> io::Result<()> {
    /*
    sudo nmcli con add type wifi ifname wlan0 con-name mobile ssid "SSID"
    sudo nmcli con modify mobile wifi-sec.key-mgmt wpa-psk
    sudo nmcli con modify mobile wifi-sec.psk "PASSWORD"
    sudo nmcli con up mobile

     */
    Ok(())
}

#[cfg(not(feature = "display_hat"))]
pub async fn add_connection(_ssid: &str, _password: &str) -> io::Result<()> {
    Ok(())
}
