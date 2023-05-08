use anyhow::Result;
use clap::Parser;
use git_meta::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    let result = args.execute()?;
    println!("{}", result);
    Ok(())
}
