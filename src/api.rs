use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone)] pub struct ArkoviaApi { client: Client, url: String }
#[derive(Debug, Deserialize)] pub struct Currency { pub currency: String, pub code: String, pub decimals: u8, pub algorithm: Option<u8> }
#[derive(Debug, Deserialize)] pub struct MintingTarget { pub counter: u64, #[serde(rename="targetBytes")] pub target_bytes: String, pub difficulty: String }

impl ArkoviaApi {
    pub fn new(url: String, timeout_seconds: u64) -> Result<Self> { Ok(Self { client: Client::builder().timeout(Duration::from_secs(timeout_seconds)).build()?, url }) }
    fn request(&self, request_type: &str, mut fields: Vec<(&str, String)>) -> Result<Value> {
        fields.push(("requestType", request_type.into()));
        let response: Value = self.client.post(&self.url).form(&fields).send().with_context(|| format!("calling {request_type}"))?.error_for_status()?.json()?;
        if let Some(code) = response.get("errorCode") { bail!("Arkovia API error {code}: {}", response.get("errorDescription").and_then(Value::as_str).unwrap_or("unknown error")); }
        Ok(response)
    }
    pub fn get_currency(&self, code: &str) -> Result<Currency> { serde_json::from_value(self.request("getCurrency", vec![("code", code.into())])?).context("invalid getCurrency response") }
    pub fn get_minting_target(&self, currency: u64, account_rs: &str, units: u64) -> Result<MintingTarget> {
        serde_json::from_value(self.request("getMintingTarget", vec![("currency", currency.to_string()), ("account", account_rs.into()), ("units", units.to_string())])?).context("invalid getMintingTarget response")
    }
    /// Creates a transaction with broadcast=false and a public key. No secret phrase is ever sent.
    pub fn prepare_mint(&self, currency: u64, nonce: u64, units: u64, counter: u64, fee_nqt: u64, public_key: &str) -> Result<Value> {
        self.request("currencyMint", vec![("currency", currency.to_string()), ("nonce", nonce.to_string()), ("units", units.to_string()), ("counter", counter.to_string()), ("feeNQT", fee_nqt.to_string()), ("deadline", "120".into()), ("publicKey", public_key.into()), ("broadcast", "false".into())])
    }
    pub fn broadcast(&self, transaction_bytes: &str) -> Result<Value> { self.request("broadcastTransaction", vec![("transactionBytes", transaction_bytes.into())]) }
    pub fn health(&self) -> Result<Value> { self.request("getBlockchainStatus", vec![]) }
}

pub fn unsigned_transaction(prepared: &Value) -> Result<&str> { prepared.get("unsignedTransactionBytes").and_then(Value::as_str).context("node did not return unsignedTransactionBytes; check public key and API permissions") }
pub fn signer_payload(prepared: &Value) -> Value { json!({ "unsignedTransactionBytes": unsigned_transaction(prepared).unwrap_or(""), "transactionJSON": prepared.get("transactionJSON").cloned().unwrap_or(Value::Null) }) }
