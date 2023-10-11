use anyhow::Result;
use rustysky::agent;
use rustysky::bsky_agent;
use rustysky::client;
use rustysky::moderation;
use rustysky::richtext;
use rustysky::types;

fn main() -> Result<()> {
    println!("Welcome to the rustysky auth example!");

    let module_name = bsky_agent::get_module_name()?;
    println!("Using module: {}", module_name);

    Ok(())
}
