use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde::de::Deserializer;
use std::{fs, path::Path};

pub const MINIMUM_FEE_NQT: u64 = 1_000_000; // 0.01 ARKOS

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub currency: CurrencyConfig,
    pub wallet: WalletConfig,
    pub miner: MinerConfig,
    pub fees: FeesConfig,
    #[serde(default)] pub signer: SignerConfig,
}

#[derive(Debug, Clone, Deserialize)] pub struct NodeConfig { pub url: String, #[serde(default = "default_timeout")] pub timeout_seconds: u64, #[serde(default)] pub explorer_url: String }
#[derive(Debug, Clone, Deserialize)] pub struct CurrencyConfig { pub name: String, pub code: String, #[serde(default)] pub id: String, #[serde(deserialize_with = "amount_as_string")] pub units_per_mint: String }
#[derive(Debug, Clone, Deserialize)] pub struct WalletConfig { pub account_rs: String, #[serde(default)] pub account_id: String, #[serde(default)] pub public_key: String }
#[derive(Debug, Clone, Deserialize)] pub struct MinerConfig { #[serde(default)] pub threads: usize, #[serde(default)] pub initial_nonce: String, #[serde(default = "default_refresh")] pub refresh_seconds: u64, #[serde(default = "default_mode")] pub submit_mode: String }
#[derive(Debug, Clone, Deserialize)] pub struct FeesConfig { pub fee_nqt: String }
#[derive(Debug, Clone, Default, Deserialize)] pub struct SignerConfig { #[serde(default)] pub command: String }
fn default_timeout() -> u64 { 20 } fn default_refresh() -> u64 { 20 } fn default_mode() -> String { "prepare".into() }

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let value: Self = toml::from_str(&fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?)?;
        Ok(value)
    }
    pub fn currency_id(&self) -> Result<Option<u64>> { if self.currency.id.trim().is_empty() || self.currency.id == "0" { Ok(None) } else { Ok(Some(self.currency.id.parse()?)) } }
    pub fn account_id(&self) -> Result<u64> { self.wallet.account_id.parse().context("wallet.account_id must be the unsigned numeric account ID") }
    pub fn fee_nqt(&self) -> Result<u64> { Ok(self.fees.fee_nqt.parse()?) }
    /// Converts a human amount (for example `0.1`) into exact atomic units without floats.
    pub fn mint_units_atomic(&self, decimals: u8) -> Result<u64> {
        let raw = self.currency.units_per_mint.trim();
        if raw.is_empty() || raw.starts_with('-') { bail!("currency.units_per_mint must be a positive decimal amount") }
        let mut parts = raw.split('.'); let whole = parts.next().unwrap_or(""); let fraction = parts.next().unwrap_or("");
        if parts.next().is_some() || (!whole.is_empty() && !whole.bytes().all(|b| b.is_ascii_digit())) || !fraction.bytes().all(|b| b.is_ascii_digit()) { bail!("currency.units_per_mint must be a plain decimal amount, such as 0.1") }
        if fraction.len() > decimals as usize { bail!("currency.units_per_mint has more than {decimals} decimal places") }
        let whole = if whole.is_empty() { "0" } else { whole }; let mut atomic = String::from(whole); atomic.push_str(fraction); atomic.push_str(&"0".repeat(decimals as usize - fraction.len()));
        let units: u64 = atomic.parse().context("currency.units_per_mint is too large")?;
        if units == 0 { bail!("currency.units_per_mint must be greater than zero") } Ok(units)
    }
    pub fn validate_node(&self) -> Result<()> { if self.node.url.trim().is_empty() { bail!("node.url is required") } Ok(()) }
    pub fn validate_mining(&self) -> Result<()> {
        self.validate_node()?;
        if self.wallet.account_rs.trim().is_empty() { bail!("wallet.account_rs is required") }
        if !self.wallet.public_key.is_empty() && (self.wallet.public_key.len() != 64 || !self.wallet.public_key.bytes().all(|b| b.is_ascii_hexdigit())) { bail!("wallet.public_key must be a 64-character hexadecimal public key") }
        if self.fee_nqt()? < MINIMUM_FEE_NQT { bail!("fee_nqt must be at least {MINIMUM_FEE_NQT} (0.01 ARKOS)") }
        if !matches!(self.miner.submit_mode.as_str(), "prepare" | "broadcast") { bail!("miner.submit_mode must be prepare or broadcast") }
        if self.miner.submit_mode == "broadcast" && self.signer.command.trim().is_empty() { bail!("broadcast mode requires a local signer.command") }
        Ok(())
    }
}

fn amount_as_string<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<String, D::Error> {
    #[derive(Deserialize)] #[serde(untagged)] enum Amount { Text(String), Whole(u64) }
    match Amount::deserialize(deserializer)? { Amount::Text(value) => Ok(value), Amount::Whole(value) => Ok(value.to_string()) }
}
