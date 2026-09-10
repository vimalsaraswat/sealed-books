//! Integration & Network Probe Harness CLI Runner
//!
//! Usage:
//!   cargo run -p harness          # runs all probes
//!   cargo run -p harness hedera   # runs only Hedera HCS probe
//!   cargo run -p harness privy    # runs only Privy wallet/signature probe

mod crypto;
mod hedera;
mod privy;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("============================================================");
    println!(" Sealed Books — Network & Integration Harness");
    println!("============================================================");

    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    let mut had_failures = false;

    if mode == "all" || mode == "hedera" {
        println!("\n>>> [1/2] Running Hedera Consensus Service Probe...");
        if let Err(e) = hedera::run_hedera_probe().await {
            eprintln!("❌ Hedera probe FAILED: {e}");
            had_failures = true;
        } else {
            println!("✅ Hedera probe SUCCEEDED!");
        }
    }

    if mode == "all" || mode == "privy" {
        println!("\n>>> [2/2] Running Privy Wallet & Signature Probe...");
        if let Err(e) = privy::run_privy_probe().await {
            eprintln!("❌ Privy probe FAILED: {e}");
            had_failures = true;
        } else {
            println!("✅ Privy probe SUCCEEDED!");
        }
    }

    println!("\n============================================================");
    if had_failures {
        eprintln!("⚠️  Harness run finished with failures. Check errors above.");
        std::process::exit(1);
    } else {
        println!("🎉 All requested harness probes passed successfully!");
    }
    println!("============================================================");

    Ok(())
}
