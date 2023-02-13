use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;
use git2::Oid;
use git2::Repository;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Configuration::from_args(&args)?;
    let result = config.execute()?;
    for (tag, oid) in result {
        println!("{tag} {oid}");
    }

    Ok(())
}

struct Configuration {
    repo: Repository,
    branch1: Oid,
    branch2: Oid,
}

impl Configuration {
    pub(crate) fn from_args(args: &Args) -> Result<Self> {
        let repo = Repository::open(&args.repo)?;
        let branch1 = repo.revparse_single(&args.target1)?.id();
        let branch2 = repo.revparse_single(&args.target2)?.id();
        Ok(Configuration {
            repo,
            branch1,
            branch2,
        })
    }

    pub(crate) fn execute(self) -> Result<Vec<(&'static str, Oid)>> {
        let base = self.repo.merge_base(self.branch1, self.branch2)?;
        let set1 = list_commits(&self.repo, base, self.branch1)?;
        let set2 = list_commits(&self.repo, base, self.branch2)?;
        Ok(set_compare(set1, set2))
    }
}

const BRANCH1_ONLY: &str = "<";
// const PROBABLY_SAME :&str = "=?";
const SAME: &str = "=";
const BRANCH2_ONLY: &str = ">";

fn set_compare(map1: HashMap<Oid, Oid>, map2: HashMap<Oid, Oid>) -> Vec<(&'static str, Oid)> {
    let mut result = Vec::new();

    for (patch_id, oid) in map1.iter() {
        if map2.contains_key(patch_id) {
            result.push((SAME, *oid))
        } else {
            result.push((BRANCH1_ONLY, *oid))
        }
    }

    for (patch_id, oid) in map2.iter() {
        if !map1.contains_key(patch_id) {
            result.push((BRANCH2_ONLY, *oid))
        }
    }

    result
}

fn get_patchid_from_commit(repo: &Repository, commit: Oid) -> Result<Oid> {
    let commit = repo.find_commit(commit)?;
    let commit_tree = commit.tree()?;
    let parent_tree = {
        let parent = commit.parent_id(0)?;
        repo.find_commit(parent)?.tree()?
    };

    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;
    let oid = diff.patchid(None)?;
    Ok(oid)
}

fn list_commits(repo: &Repository, base: Oid, head: Oid) -> Result<HashMap<Oid, Oid>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.hide(base)?;
    revwalk.push(head)?;
    revwalk.simplify_first_parent()?;

    let mut res = HashMap::new();
    for rev in revwalk {
        let oid = rev?;
        res.insert(get_patchid_from_commit(repo, oid)?, oid);
    }

    Ok(res)
}

#[derive(Parser)]
#[command(author, version, about = "Compare two branches", long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".", help = "Path to git repository")]
    repo: String,

    #[arg(
        default_value = "HEAD",
        help = "First target. sha, short-sha, refs are all valid input"
    )]
    target1: String,

    #[arg(
        default_value = "HEAD",
        help = "Second target. sha, short-sha, refs are all valid input"
    )]
    target2: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foo() {
        let a = Args {
            repo: String::from("/home/jo/backend"),
            target1: String::from("origin/release/230215"),
            target2: String::from("HEAD"),
        };

        let c = Configuration::from_args(&a).unwrap();
        c.execute().unwrap();
    }
}
