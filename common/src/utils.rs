use anyhow::Result;

pub fn shake_your_hash() -> Result<String> {
    Ok(sha256::digest(
        std::fs::read(std::env::current_exe()?)?.as_slice(),
    ))
}
