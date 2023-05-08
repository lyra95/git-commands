use anyhow::Result;
use clap::Parser;
use git_compare::WhichBranch;

fn main() -> Result<()> {
    let args = Args::parse();
    let commits = git_compare::Args {
        repo: args.repo.clone(),
        target1: args.branch1.clone(),
        target2: args.branch2.clone(),
    }
    .execute()?;

    let mut wtr = {
        let mut builder = csv::WriterBuilder::new();
        if args.print_header {
            builder.has_headers(true);
        }
        builder.double_quote(true);
        builder.from_writer(vec![])
    };

    if args.print_header {
        wtr.write_record(&[
            "which_branch",
            "author",
            "summary",
            "message",
            "email",
            "jira",
        ])?;
    }

    for commit in commits {
        let which_branch = match commit.0 {
            WhichBranch::Branch1Only => args.branch1.clone(),
            WhichBranch::Branch2Only => args.branch2.clone(),
            WhichBranch::Both => "Both".to_string(),
        };
        let parsed = git_meta::Args {
            repo: args.repo.clone(),
            commit: commit.1.to_string(),
            format: r#"${author},${summary},${message},${email},${jira}"#.to_string(),
        }
        .get_info()?;

        wtr.write_record(&[
            &which_branch,
            parsed.get("author").unwrap(),
            parsed.get("summary").unwrap(),
            parsed.get("message").unwrap(),
            parsed.get("email").unwrap(),
            parsed.get("jira").unwrap(),
        ])?;
    }

    print!("{}", String::from_utf8(wtr.into_inner()?)?);
    Ok(())
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = ".", help = "Path to git repository")]
    repo: String,

    #[arg(long, help = "first branch, ex: origin/main")]
    branch1: String,

    #[arg(long, help = "second branch, ex: origin/release/230518")]
    branch2: String,

    #[arg(long, help = "print csv header")]
    print_header: bool,
}
