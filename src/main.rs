// SPDX-FileCopyrightText: 2026 Andrew Gunnerson
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use hmac::{Hmac, KeyInit, Mac};
use libcryptsetup_rs::CryptInit;
use sha2::Sha256;
use uuid::Uuid;
use zeroize::Zeroizing;

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Compute the LUKS volume key hash for use with systemd's fixate-volume-key
/// crypttab option.
#[derive(Debug, Parser)]
pub struct Cli {
    /// The device-mapper name used when attaching the device.
    ///
    /// Defaults to "luks-<lowercase UUID>".
    #[arg(short, long)]
    name: Option<String>,

    /// LUKS block device or file image.
    path: PathBuf,
}

fn compute_message(uuid: Uuid, name: Option<&str>) -> String {
    let volume_name = match name {
        Some(n) => {
            let mut buf = String::with_capacity(n.len() * 2);

            for c in n.chars() {
                if !(' '..'\x7f').contains(&c) || c == '\\' || c == ':' {
                    buf.push_str("\\x");
                    buf.push(HEX[usize::from(c as u8 >> 4)]);
                    buf.push(HEX[usize::from(c as u8 & 0xf)]);
                } else {
                    buf.push(c);
                }
            }

            buf
        }
        None => format!("luks-{uuid}"),
    };

    format!("cryptsetup:{volume_name}:{uuid}")
}

fn compute_hash(volume_key: &[u8], message: &str) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(volume_key).expect("HMAC-SHA-256 has no key size limit");

    mac.update(message.as_bytes());

    let result = mac.finalize();
    let mut hex_result = String::with_capacity(result.as_bytes().len() * 2);

    for b in result.as_bytes() {
        hex_result.push(HEX[usize::from(*b >> 4)]);
        hex_result.push(HEX[usize::from(*b & 0xf)]);
    }

    hex_result
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut device = CryptInit::init(&cli.path)
        .with_context(|| format!("Failed to open device: {:?}", cli.path))?;

    device
        .context_handle()
        .load::<()>(None, None)
        .context("Failed to load volume header")?;

    let uuid = device
        .status_handle()
        .get_uuid()
        .context("Failed to get volume UUID")?;
    let message = compute_message(uuid, cli.name.as_deref());

    let volume_key_size = device.status_handle().get_volume_key_size();
    let mut volume_key = Zeroizing::new(vec![0u8; volume_key_size as usize]);

    let passphrase = Zeroizing::new(passterm::prompt_password_tty(Some("Passphrase: "))?);

    device
        .volume_key_handle()
        .get(None, &mut volume_key, Some(passphrase.as_bytes()))
        .context("Failed to get volume key (wrong passphrase?)")?;

    let hash = compute_hash(&volume_key, &message);

    println!("{hash}");

    Ok(())
}
