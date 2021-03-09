use std::process::Command;

pub struct GitNoteRepo {
    git_dir: String,
    indexed_commit: String, // Last indexed commit SHA1
}

impl GitNoteRepo {
    pub fn list_changes(&self) -> anyhow::Result<&str> {
        let output = Command::new("git").arg("status").output()?;
        return Ok("win");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}