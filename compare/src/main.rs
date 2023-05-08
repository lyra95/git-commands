use anyhow::Result;
use clap::Parser;
use git_compare::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    let result = args.execute()?;
    for (tag, oid) in result {
        println!("{:?} {}", tag, oid);
    }
    Ok(())
}
