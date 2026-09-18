mod api; mod config; mod protocol;

use anyhow::{bail, Context, Result};
use api::{ArkoviaApi, signer_payload};
use clap::{Parser, Subcommand};
use config::Config;
use rand::Rng;
use std::{path::PathBuf, process::{Command, Stdio}, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc}, thread, time::{Duration, Instant}};

#[derive(Parser)] #[command(name="arkovia-miner", version, about="Arkovia Scrypt currency miner")]
struct Cli { #[arg(short, long, default_value="miner.toml")] config: PathBuf, #[command(subcommand)] command: Commands }
#[derive(Subcommand)] enum Commands { Mine, Status, Init }

fn main() -> Result<()> { let cli=Cli::parse(); match cli.command { Commands::Init => { std::fs::copy("miner.example.toml", &cli.config).context("copying example configuration")?; println!("Created {}. Edit it before mining.", cli.config.display()); Ok(()) }, Commands::Status => status(&Config::load(&cli.config)?), Commands::Mine => mine(Config::load(&cli.config)?) } }

fn banner() { println!("\x1b[92m\n █████╗ ██████╗ ██╗  ██╗ ██████╗ ██╗   ██╗██╗ █████╗ \n██╔══██╗██╔══██╗██║ ██╔╝██╔═══██╗██║   ██║██║██╔══██╗\n███████║██████╔╝█████╔╝ ██║   ██║██║   ██║██║███████║\n██╔══██║██╔══██╗██╔═██╗ ██║   ██║╚██╗ ██╔╝██║██╔══██║\n██║  ██║██║  ██║██║  ██╗╚██████╔╝ ╚████╔╝ ██║██║  ██║\n╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═══╝  ╚═╝╚═╝  ╚═╝\n                 B L O C K C H A I N\x1b[0m\n"); }
fn status(config: &Config) -> Result<()> { banner(); let api=ArkoviaApi::new(config.node.url.clone(),config.node.timeout_seconds)?; let health=api.health()?; println!("Connection: ONLINE\nBlockchain: {}\nHeight: {}",health.get("application").and_then(|v|v.as_str()).unwrap_or("Arkovia"),health.get("numberOfBlocks").and_then(|v|v.as_u64()).unwrap_or(0)); Ok(()) }

fn mine(config: Config) -> Result<()> {
    banner(); let api=ArkoviaApi::new(config.node.url.clone(),config.node.timeout_seconds)?;
    let currency=api.get_currency(&config.currency.code)?; let currency_id=config.currency_id()?.unwrap_or(currency.currency.parse()?);
    if currency.algorithm != Some(5) { bail!("{} ({}) is not a Scrypt minting currency (expected algorithm ID 5)", currency.code, currency.currency); }
    let decimals=currency.decimals; let multiplier=10u64.checked_pow(decimals as u32).context("unsupported currency decimals")?; let units=config.currency.units_per_mint.checked_mul(multiplier).context("units overflow")?;
    let account_id=config.account_id()?; let threads=if config.miner.threads==0 { num_cpus::get() } else { config.miner.threads }; let fee=config.fee_nqt()?;
    println!("\x1b[92mCurrency: {} ({}) | ID: {} | Algorithm: Scrypt\nAccount: {} | Units: {} | CPU threads: {} | Fee: {} NQT\x1b[0m", config.currency.name,currency.code,currency_id,config.wallet.account_rs,units,threads,fee);
    loop {
        let target=api.get_minting_target(currency_id,&config.wallet.account_rs,units)?; let target_bytes: [u8;32]=hex::decode(&target.target_bytes).context("invalid targetBytes hex")?.try_into().map_err(|_| anyhow::anyhow!("targetBytes must contain 32 bytes"))?;
        println!("\x1b[92mConnection: ONLINE | Difficulty: {} | Counter: {} | searching…\x1b[0m",target.difficulty,target.counter);
        let start_nonce=match config.miner.initial_nonce.parse::<u64>().unwrap_or(0) { 0 => rand::thread_rng().gen(), n => n };
        let (nonce, attempts, elapsed)=solve(currency_id,account_id,units,target.counter,target_bytes,start_nonce,threads)?;
        println!("\x1b[92mSolution found: nonce {} | {:.2} H/s | {} hashes\x1b[0m",nonce,attempts as f64/elapsed.as_secs_f64(),attempts);
        println!("Preparing locally-signable mint transaction; no secret phrase is sent to the node.");
        // A public key is intentionally entered by the user at runtime. It lets the node create unsigned bytes only.
        let public_key=read_line("Enter 64-character public key (or press Enter to continue mining without preparing): ")?;
        if public_key.trim().is_empty() { continue; }
        let prepared=api.prepare_mint(currency_id,nonce,units,target.counter,fee,public_key.trim())?;
        if config.miner.submit_mode=="prepare" { println!("Unsigned mint transaction:\n{}",serde_json::to_string_pretty(&signer_payload(&prepared))?); }
        else { let signed=run_signer(&config.signer.command,&signer_payload(&prepared))?; println!("Broadcast result: {}",serde_json::to_string_pretty(&api.broadcast(&signed)?)?); }
    }
}

fn solve(currency_id:u64, account_id:u64, units:u64, counter:u64, target:[u8;32], start:u64, threads:usize)->Result<(u64,u64,Duration)> {
    let found=Arc::new(AtomicBool::new(false)); let attempts=Arc::new(AtomicU64::new(0)); let solution=Arc::new(AtomicU64::new(0)); let began=Instant::now(); let mut handles=Vec::new();
    for index in 0..threads { let found=found.clone();let attempts=attempts.clone();let solution=solution.clone(); handles.push(thread::spawn(move || { let mut nonce=start.wrapping_add(index as u64); while !found.load(Ordering::Relaxed) { let hash=protocol::scrypt_hash(&protocol::work_bytes(nonce,currency_id,units,counter,account_id)); attempts.fetch_add(1,Ordering::Relaxed); if protocol::meets_target(&hash,&target) { solution.store(nonce,Ordering::Relaxed);found.store(true,Ordering::Relaxed);break;} nonce=nonce.wrapping_add(threads as u64); } })); }
    for handle in handles { handle.join().map_err(|_|anyhow::anyhow!("mining thread panicked"))?; } Ok((solution.load(Ordering::Relaxed),attempts.load(Ordering::Relaxed),began.elapsed()))
}
fn read_line(prompt:&str)->Result<String>{use std::io::{self,Write};print!("{prompt}");io::stdout().flush()?;let mut line=String::new();io::stdin().read_line(&mut line)?;Ok(line)}
fn run_signer(command:&str,payload:&serde_json::Value)->Result<String>{use std::io::Write;let mut child=Command::new("sh").arg("-c").arg(command).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().context("starting local signer")?;child.stdin.as_mut().context("opening signer stdin")?.write_all(serde_json::to_string(payload)?.as_bytes())?;let output=child.wait_with_output()?;if !output.status.success(){bail!("local signer failed")};let signed=String::from_utf8(output.stdout)?.trim().to_owned();if signed.is_empty(){bail!("local signer returned no transaction bytes")};Ok(signed)}
