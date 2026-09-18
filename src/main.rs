mod api; mod config; mod protocol;

use anyhow::{bail, Context, Result};
use api::{ArkoviaApi, signer_payload};
use clap::{Parser, Subcommand};
use config::Config;
use rand::Rng;
use std::{fs, io::{self, Write}, path::PathBuf, process::{Command, Stdio}, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc}, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

#[derive(Parser)] #[command(name="arkovia-miner", version, about="Arkovia Scrypt currency miner")]
struct Cli { #[arg(short, long, default_value="miner.toml")] config: PathBuf, #[command(subcommand)] command: Commands }
#[derive(Subcommand)] enum Commands { Mine, Status, Init { #[arg(long)] interactive: bool, #[arg(long)] force: bool } }

fn main() -> Result<()> { let cli=Cli::parse(); match cli.command { Commands::Init { interactive, force } => init(&cli.config, interactive, force), Commands::Status => status(&Config::load(&cli.config)?), Commands::Mine => mine(Config::load(&cli.config)?) } }

fn banner() { println!("\x1b[92m\n █████╗ ██████╗ ██╗  ██╗ ██████╗ ██╗   ██╗██╗ █████╗ \n██╔══██╗██╔══██╗██║ ██╔╝██╔═══██╗██║   ██║██║██╔══██╗\n███████║██████╔╝█████╔╝ ██║   ██║██║   ██║██║███████║\n██╔══██║██╔══██╗██╔═██╗ ██║   ██║╚██╗ ██╔╝██║██╔══██║\n██║  ██║██║  ██║██║  ██╗╚██████╔╝ ╚████╔╝ ██║██║  ██║\n╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝   ╚═══╝  ╚═╝╚═╝  ╚═╝\n                 B L O C K C H A I N\x1b[0m\n"); }
fn init(path:&std::path::Path, interactive:bool, force:bool)->Result<()> {
    if path.exists() && !force { bail!("{} already exists. Use --force to replace it.",path.display()) }
    if !interactive { fs::copy("miner.example.toml",path).context("copying example configuration")?; println!("Created {}. Edit it before mining, or rerun with init --interactive.",path.display()); return Ok(()); }
    banner(); println!("\x1b[92mFirst-run setup — secret phrases are never requested or stored.\x1b[0m");
    let node=ask("Arkovia node URL", "https://arkovia-node1.mywire.org/nxt")?;
    let name=ask("Currency name", "Meldralite")?;
    let code=ask("Currency code", "MLT")?;
    let account_rs=ask("Your Arkovia RS account", "")?;
    let account_id=ask("Your numeric account ID", "")?;
    let public_key=ask("Your 64-character public key", "")?;
    let units=ask("Whole units per mint", "1")?;
    let threads=ask("CPU threads (0 = all)", "0")?;
    let config=format!("# Created by the Arkovia Universal Currency Miner setup wizard.\n[node]\nurl = \"{}\"\ntimeout_seconds = 20\nexplorer_url = \"\"\n\n[currency]\nname = \"{}\"\ncode = \"{}\"\nid = \"0\"\nunits_per_mint = {}\n\n[wallet]\naccount_rs = \"{}\"\naccount_id = \"{}\"\npublic_key = \"{}\"\n\n[miner]\nthreads = {}\ninitial_nonce = \"0\"\nrefresh_seconds = 20\nsubmit_mode = \"prepare\"\n\n[fees]\nfee_nqt = \"1000000\"\n\n[signer]\ncommand = \"\"\n",quote(&node),quote(&name),quote(&code),units.trim(),quote(&account_rs),quote(&account_id),quote(&public_key),threads.trim());
    fs::write(path,config)?; let checked=Config::load(path)?; checked.validate_mining()?; println!("\x1b[92mCreated {}. Run `status` first, then mine in prepare mode.\x1b[0m",path.display()); Ok(())
}
fn ask(label:&str, default:&str)->Result<String>{ if default.is_empty(){print!("{label}: ");}else{print!("{label} [{default}]: ");}io::stdout().flush()?;let mut value=String::new();io::stdin().read_line(&mut value)?;let value=value.trim().to_owned();Ok(if value.is_empty(){default.to_owned()}else{value}) }
fn quote(value:&str)->String { value.replace('\\',"\\\\").replace('"',"\\\"") }
fn status(config: &Config) -> Result<()> { config.validate_node()?; banner(); let api=ArkoviaApi::new(config.node.url.clone(),config.node.timeout_seconds)?; let health=api.health()?; println!("Connection: ONLINE\nBlockchain: {}\nHeight: {}",health.get("application").and_then(|v|v.as_str()).unwrap_or("Arkovia"),health.get("numberOfBlocks").and_then(|v|v.as_u64()).unwrap_or(0)); Ok(()) }

fn mine(config: Config) -> Result<()> {
    config.validate_mining()?; banner(); let api=ArkoviaApi::new(config.node.url.clone(),config.node.timeout_seconds)?;
    let currency=api.get_currency(&config.currency.code)?; let currency_id=config.currency_id()?.unwrap_or(currency.currency.parse()?);
    if currency.algorithm != Some(5) { bail!("{} ({}) is not a Scrypt minting currency (expected algorithm ID 5)", currency.code, currency.currency); }
    let decimals=currency.decimals; let multiplier=10u64.checked_pow(decimals as u32).context("unsupported currency decimals")?; let units=config.currency.units_per_mint.checked_mul(multiplier).context("units overflow")?;
    let account_id=config.account_id()?; let threads=if config.miner.threads==0 { num_cpus::get() } else { config.miner.threads }; let fee=config.fee_nqt()?;
    println!("\x1b[92mCurrency: {} ({}) | ID: {} | Algorithm: Scrypt\nAccount: {} | Units: {} | CPU threads: {} | Fee: {} NQT\x1b[0m", config.currency.name,currency.code,currency_id,config.wallet.account_rs,units,threads,fee);
    loop {
        let target=api.get_minting_target(currency_id,&config.wallet.account_rs,units)?; let target_bytes: [u8;32]=hex::decode(&target.target_bytes).context("invalid targetBytes hex")?.try_into().map_err(|_| anyhow::anyhow!("targetBytes must contain 32 bytes"))?;
        println!("\x1b[92mConnection: ONLINE | Difficulty: {} | Counter: {} | searching…\x1b[0m",target.difficulty,target.counter);
        let start_nonce=match config.miner.initial_nonce.parse::<u64>().unwrap_or(0) { 0 => rand::thread_rng().gen(), n => n };
        let result=solve(currency_id,account_id,units,target.counter,target_bytes,start_nonce,threads,Duration::from_secs(config.miner.refresh_seconds))?;
        let Some((nonce, attempts, elapsed))=result else { println!("\x1b[93mTarget refresh interval reached; requesting fresh work.\x1b[0m"); continue; };
        println!("\x1b[92mSolution found: nonce {} | {:.2} H/s | {} hashes\x1b[0m",nonce,attempts as f64/elapsed.as_secs_f64(),attempts);
        println!("Preparing locally-signable mint transaction; no secret phrase is sent to the node.");
        if config.wallet.public_key.trim().is_empty() { println!("\x1b[93mNo wallet.public_key configured. Solution was not prepared for submission.\x1b[0m"); continue; }
        let prepared=api.prepare_mint(currency_id,nonce,units,target.counter,fee,&config.wallet.public_key)?;
        if config.miner.submit_mode=="prepare" { save_prepared_transaction(&prepared)?; }
        else { let signed=run_signer(&config.signer.command,&signer_payload(&prepared))?; print_broadcast(&api.broadcast(&signed)?,&config.node.explorer_url); }
    }
}

fn solve(currency_id:u64, account_id:u64, units:u64, counter:u64, target:[u8;32], start:u64, threads:usize, refresh_after:Duration)->Result<Option<(u64,u64,Duration)>> {
    let stop=Arc::new(AtomicBool::new(false)); let found=Arc::new(AtomicBool::new(false)); let attempts=Arc::new(AtomicU64::new(0)); let solution=Arc::new(AtomicU64::new(0)); let began=Instant::now(); let mut handles=Vec::new();
    let reporter_stop=stop.clone(); let reporter_attempts=attempts.clone(); let reporter_began=began;
    let reporter=thread::spawn(move || { while !reporter_stop.load(Ordering::Relaxed) { thread::sleep(Duration::from_secs(1)); let elapsed=reporter_began.elapsed().as_secs_f64(); if elapsed > 0.0 { println!("\x1b[92mHash rate: {:.2} H/s | Attempts: {}\x1b[0m",reporter_attempts.load(Ordering::Relaxed) as f64/elapsed,reporter_attempts.load(Ordering::Relaxed)); } } });
    for index in 0..threads { let stop=stop.clone(); let found=found.clone();let attempts=attempts.clone();let solution=solution.clone(); handles.push(thread::spawn(move || { let mut nonce=start.wrapping_add(index as u64); while !stop.load(Ordering::Relaxed) { let hash=protocol::scrypt_hash(&protocol::work_bytes(nonce,currency_id,units,counter,account_id)); attempts.fetch_add(1,Ordering::Relaxed); if protocol::meets_target(&hash,&target) { solution.store(nonce,Ordering::Relaxed);found.store(true,Ordering::Relaxed);stop.store(true,Ordering::Relaxed);break;} nonce=nonce.wrapping_add(threads as u64); } })); }
    while !stop.load(Ordering::Relaxed) { if began.elapsed() >= refresh_after { stop.store(true,Ordering::Relaxed); } else { thread::sleep(Duration::from_millis(100)); } }
    for handle in handles { handle.join().map_err(|_|anyhow::anyhow!("mining thread panicked"))?; } reporter.join().map_err(|_|anyhow::anyhow!("status reporter panicked"))?;
    let elapsed=began.elapsed(); if found.load(Ordering::Relaxed) { Ok(Some((solution.load(Ordering::Relaxed),attempts.load(Ordering::Relaxed),elapsed))) } else { Ok(None) }
}
fn save_prepared_transaction(prepared:&serde_json::Value)->Result<()> { let payload=signer_payload(prepared); let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(); let path=format!("prepared-mint-{timestamp}.json"); fs::write(&path,serde_json::to_string_pretty(&payload)?)?; println!("\x1b[92mUnsigned mint transaction saved locally: {path}\x1b[0m"); Ok(()) }
fn print_broadcast(response:&serde_json::Value, explorer_url:&str) { let transaction=response.get("transaction").and_then(|v|v.as_str()).unwrap_or("unknown"); let full_hash=response.get("fullHash").and_then(|v|v.as_str()).unwrap_or("unknown"); println!("\x1b[92mMint accepted by node. Transaction: {transaction}\nFull hash: {full_hash}\x1b[0m"); if !explorer_url.trim().is_empty() && transaction!="unknown" { println!("Explorer: {}/transaction/{}",explorer_url.trim_end_matches('/'),transaction); } }
fn run_signer(command:&str,payload:&serde_json::Value)->Result<String>{use std::io::Write;
    #[cfg(target_os="windows")] let mut child=Command::new("cmd").arg("/C").arg(command).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().context("starting local signer")?;
    #[cfg(not(target_os="windows"))] let mut child=Command::new("sh").arg("-c").arg(command).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().context("starting local signer")?;
    child.stdin.as_mut().context("opening signer stdin")?.write_all(serde_json::to_string(payload)?.as_bytes())?;let output=child.wait_with_output()?;if !output.status.success(){bail!("local signer failed")};let signed=String::from_utf8(output.stdout)?.trim().to_owned();if signed.is_empty(){bail!("local signer returned no transaction bytes")};if !signed.bytes().all(|b|b.is_ascii_hexdigit()){bail!("local signer did not return hexadecimal transaction bytes")};Ok(signed)}
