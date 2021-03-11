use std::process::Command;
use std::path;

use git2::Repository;

pub struct GitNoteRepo {
    git_dir: path::PathBuf,
    indexed_commit: Option<String>, // Last indexed commit SHA1
}

impl GitNoteRepo {

    fn git(&self) -> std::process::Command {
        let mut git = Command::new("git");
        git.arg("-C");
        git.arg(self.git_dir.as_os_str());
        git
    }

    pub fn list_changes(&self) -> anyhow::Result<&str> {
        let mut command = self.git();

        if let Some(hash) = &self.indexed_commit {
            command.arg("diff-tree");
            command.arg(hash);
            command.arg("HEAD");
        } else {
            command.arg("ls-tree");
            command.arg("HEAD");
        }

        command
            .arg("-r")
            .arg("--name-status")
            .arg("--full-name");

        let output = command.output()?;

        print!("{:?}", output);
        return Ok("win");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use anyhow::Context;
    use std::io::Write;
    use git2::{Repository,Signature};

    #[test]
    fn test_list_changes() -> anyhow::Result<()> {
        let testnotes_dir = tempdir().context("failed to create tempdir")?;
        let a = testnotes_dir.path().join("a");
        let b = testnotes_dir.path().join("b");
        let c = testnotes_dir.path().join("c");
        std::fs::File::create(&a)?;
        let mut b = std::fs::File::create(b)?;
        let mut c = std::fs::File::create(c)?;

        let repo = Repository::init(testnotes_dir.path())?;
        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("a"))?;
        let tree_id = index.write_tree()?;
        index.write()?;

        let tree = repo.find_tree(tree_id)?;

        let author = Signature::now("you", "you@us")?;
        let committer = Signature::now("me", "me@us")?;
        let message = "Initial commit";

        let mut parents: Vec<&git2::Commit<'_>> = vec!();
        let head: git2::Commit<'_>;

        if let Ok(commit) = repo.refname_to_id("HEAD").and_then(|oid| repo.find_commit(oid)) {
            head = commit;
            parents.push(&head);
        };

        repo.commit(Some("HEAD"), &author, &committer, message, &tree, &parents)?;

        println!("{:?}", testnotes_dir.path());
        std::mem::forget(testnotes_dir);
        assert!(false);
        Ok(())
    }

    fn _test_list_changes() -> anyhow::Result<()> {
        let testnotes_dir = tempdir().context("failed to create tempdir")?;
        let gnr = GitNoteRepo { git_dir: testnotes_dir.path().to_owned(), indexed_commit: None };
        let a = testnotes_dir.path().join("a");
        let b = testnotes_dir.path().join("b");
        let c = testnotes_dir.path().join("c");
        let mut a = std::fs::File::create(a)?;
        let mut b = std::fs::File::create(b)?;
        let mut c = std::fs::File::create(c)?;

        write!(a, "wat");
        write!(b, "wat");
        write!(c, "wat");
        let mut sh = Command::new("sh")
            .current_dir(&testnotes_dir)
            .stdin(std::process::Stdio::piped())
            .arg("-c")
            .arg(format!("pushd {} && git init; 
                git add a && git commit -m \"add a\";
                git add b && git commit -m \"add b\";
                git add c && git commit -m \"add c\";
                ", &testnotes_dir.path().to_str().unwrap()))
            .output()
            .expect("failed to execute process");
        
        gnr.list_changes()?;
        //write!(stdin, "pushd {} && git init && git add a && git commit -m \"add a\";", &testnotes_dir.path().to_str().unwrap());
        

        gnr.list_changes()?;
        assert!(false);

        //drop(testnotes_dir);

        Ok(())
    }
}