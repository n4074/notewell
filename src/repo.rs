use anyhow::{Result, Context};
use std::path::{Path,PathBuf};

use std::io::{Write};

use git2::{Object, Repository, Delta, Commit};
use git2;

pub struct Repo {
    repo: Repository,
}

const NB_SUBDIR: &'static str = ".nb";

impl<'repo> Repo {

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Repo> {
        log::debug!("opening repository at: {:?}", path.as_ref());
        let repo = Repository::open(path).context("Failed to open git repository")?;

        Ok(Repo { repo })
    }

    pub fn init<P: AsRef<Path>>(path: P) -> Result<Repo> {
        let repo = Repository::init(path)?;
        let mut this = Repo { repo };
        this.repo.add_ignore_rule(".gitgnore")?;
        this.repo.add_ignore_rule(NB_SUBDIR)?;
        
        let workdir = this.repo.workdir()
            .context("Could not obtain work directory")?;
        
        let gitignore = Path::new(".gitignore");

        let mut output = std::fs::File::create(&workdir.join(gitignore))?;
        write!(output, "{}", NB_SUBDIR)?;

        this.commit_paths(vec!(&gitignore))?;

        Ok(this)
    }

    pub fn commit_paths<P: AsRef<Path>>(&mut self, paths: Vec<P>) -> Result<()> {
        let mut index = self.repo.index()?;

        for path in paths {
            index.add_path(path.as_ref())?;
        }

        let tree_id = index.write_tree()?;
        index.write()?;

        let tree = self.repo.find_tree(tree_id)?;

        let mut parents: Vec<&git2::Commit<'_>> = vec!();

        let head = self.head();

        if head.is_ok() {
            parents.push(head.as_ref().unwrap());
        }

        let signature = self.repo.signature()?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &"auto",
            &tree,
            &parents
        )?;

        Ok(())
    }

    pub fn resolve(&self, rfn: &str) -> Result<Object<'_>> {
        self.repo.revparse_single(rfn).context(format!("Failed to resolve reference: {}", rfn))
    }

    pub fn diff(&self, old: Option<&String>, new: Option<&String>) -> Result<Vec<(Delta, PathBuf)>> {

        let old = if let Some(old) = old {
            Some(self.resolve(&old)?.peel_to_commit()?.tree()?)
        } else {
            None
        };

        log::debug!("old_commit: {:?}", old);

        let new = if let Some(new) = new {
            Some(self.resolve(&new)?.peel_to_commit()?.tree()?)
        } else {
            Some(self.head()?.tree()?)
        };

        log::debug!("new_commit: {:?}", old);

        // If no head, we have nothing to index
        let diff = self.repo.diff_tree_to_tree(old.as_ref(), new.as_ref(), None)?;

        Ok(diff.deltas().map(|delta| {
            let path = delta.new_file().path().unwrap().to_owned();
            (delta.status(), path)
        }).collect())
    }

    pub fn head(&'repo self) -> anyhow::Result<Commit<'repo>> {
        self.repo.head()
            .and_then(|h| h.peel_to_commit())
            .context("Failed to find HEAD")
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
        let d = testnotes_dir.path().join("d");
        let _a = std::fs::File::create(&a)?;
        let _b = std::fs::File::create(b)?;
        let mut c = std::fs::File::create(c)?;
        let _d = std::fs::File::create(d)?;

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
        let mut head: git2::Commit<'_>;


        if let Ok(commit) = repo.refname_to_id("HEAD").and_then(|oid| repo.find_commit(oid)) {
            head = commit;
            parents.push(&head);
        };

        let repo2 = Repository::open(testnotes_dir.path())?;
        let nb_none = Repo { repo: repo2 };

        repo.commit(Some("HEAD"), &author, &committer, message, &tree, &parents)?;

        let oid = repo.head()?.peel_to_commit()?.id();

        let repo2 = Repository::open(testnotes_dir.path())?;
        let _nb = Repo { repo: repo2 };


        let mut index = repo.index()?;
        index.add_path(std::path::Path::new("b"))?;
        index.add_path(std::path::Path::new("c"))?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        index.write()?;

        head = repo.refname_to_id("HEAD").and_then(|oid| repo.find_commit(oid))?;
        repo.commit(Some("HEAD"), &author, &committer, message, &tree, &[&head])?;

        let mut index = repo.index()?;
        index.remove_path(std::path::Path::new("b"))?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        index.write()?;

        head = repo.refname_to_id("HEAD").and_then(|oid| repo.find_commit(oid))?;
        repo.commit(Some("HEAD"), &author, &committer, message, &tree, &[&head])?;


        let mut index = repo.index()?;
        c.write("wat".as_bytes())?;
        index.add_path(std::path::Path::new("c"))?;
        index.add_path(std::path::Path::new("d"))?;
        let tree_id = index.write_tree()?;
        let _tree = repo.find_tree(tree_id)?;
        index.write()?;


        let repo3 = Repository::open(testnotes_dir.path())?;
        let nb = Repo { repo: repo3, };
        println!("{:?}", nb.diff(Some(&oid.to_string()), None)?);
        println!("{:?}", nb_none.diff(Some(&oid.to_string()), None)?);

        println!("{:?}\n", testnotes_dir.path());


        for reference in repo.references()?.names() {
            print!("ref: {:?}\n", reference);
        }

        print!("head: {:?}\n", repo.revparse_single("HEAD")?.peel_to_tree());
        print!("head~3: {:?}\n", repo.revparse_single("HEAD~2")?.peel_to_tree());

        let tree_a = repo.revparse_single("HEAD")?.peel_to_tree()?;
        let diff = repo.diff_tree_to_workdir_with_index(Some(&tree_a), None)?;

        print!("Deltas: {:?}\n", diff.deltas().len());

        for delta in diff.deltas() {
            println!("{:?}\n", delta);
        }

        //assert!(false);
        Ok(())
    }

}