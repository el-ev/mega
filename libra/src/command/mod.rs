pub mod add;
pub mod branch;
pub mod clone;
pub mod commit;
pub mod diff;
pub mod fetch;
pub mod index_pack;
pub mod init;
pub mod lfs;
pub mod log;
pub mod merge;
pub mod pull;
pub mod push;
pub mod remote;
pub mod remove;
pub mod restore;
pub mod status;
pub mod switch;
pub mod config;

use crate::internal::branch::Branch;
use crate::internal::head::Head;
use crate::internal::protocol::https_client::BasicAuth;
use crate::utils;
use crate::utils::object_ext::BlobExt;
use crate::utils::util;
use mercury::internal::object::blob::Blob;
use mercury::{errors::GitError, hash::SHA1, internal::object::ObjectTrait};
use rpassword::read_password;
use std::io;
use std::io::Write;
use std::path::Path;

const HEAD: &str = "HEAD";

// impl load for all objects
fn load_object<T>(hash: &SHA1) -> Result<T, GitError>
where
    T: ObjectTrait,
{
    let storage = util::objects_storage();
    let data = storage.get(hash)?;
    T::from_bytes(&data.to_vec(), *hash)
}

// impl save for all objects
fn save_object<T>(object: &T, ojb_id: &SHA1) -> Result<(), GitError>
where
    T: ObjectTrait,
{
    let storage = util::objects_storage();
    let data = object.to_data()?;
    storage.put(ojb_id, &data, object.get_type())?;
    Ok(())
}

/// Ask for username and password (CLI interaction)
fn ask_username_password() -> (String, String) {
    print!("username: ");
    // Normally your OS will buffer output by line when it's connected to a terminal,
    // which is why it usually flushes when a newline is written to stdout.
    io::stdout().flush().unwrap(); // ensure the prompt is shown
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    username = username.trim().to_string();
    tracing::debug!("username: {}", username);

    print!("password: ");
    io::stdout().flush().unwrap();
    let password = if std::env::var("LIBRA_NO_HIDE_PASSWORD").is_ok() {
        // for test
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        password = password.trim().to_string();
        tracing::debug!("password: {}", password);
        password
    } else {
        // error in test environment: "No such device or address"
        read_password().unwrap() // hide password
    };
    (username, password)
}

/// same as ask_username_password, but return BasicAuth
pub fn ask_basic_auth() -> BasicAuth {
    let (username, password) = ask_username_password();
    BasicAuth { username, password }
}

/// Calculate the hash of a file blob
/// - for `lfs` file: calculate hash of the pointer data
pub fn calc_file_blob_hash(path: impl AsRef<Path>) -> io::Result<SHA1> {
    let blob = if utils::lfs::is_lfs_tracked(&path) {
        let (pointer, _) = utils::lfs::generate_pointer_file(&path);
        Blob::from_content(&pointer)
    } else {
        Blob::from_file(&path)
    };
    Ok(blob.id)
}

/// Get the commit hash from branch name or commit hash, support remote branch
pub async fn get_target_commit(branch_or_commit: &str) -> Result<SHA1, Box<dyn std::error::Error>> {
    if branch_or_commit == HEAD {
        return Ok(Head::current_commit().await.unwrap());
    }

    let possible_branches = Branch::search_branch(branch_or_commit).await;
    if possible_branches.len() > 1 {
        return Err("Ambiguous branch name".into());
        // TODO: git have a priority list of branches to use, continue with ambiguity, we didn't implement it yet
    }

    if possible_branches.is_empty() {
        let storage = util::objects_storage();
        let possible_commits = storage.search(branch_or_commit);
        if possible_commits.len() > 1 {
            return Err(format!("Ambiguous commit hash '{}'", branch_or_commit).into());
        }
        if possible_commits.is_empty() {
            return Err(format!("No such branch or commit: '{}'", branch_or_commit).into());
        }
        Ok(possible_commits[0])
    } else {
        Ok(possible_branches[0].commit)
    }
}

#[cfg(test)]
mod test {
    use common::utils::{format_commit_msg, parse_commit_msg};
    use mercury::internal::object::commit::Commit;

    use super::*;
    use crate::utils::test;
    #[tokio::test]
    async fn test_save_load_object() {
        test::setup_with_new_libra().await;
        let object = Commit::from_tree_id(SHA1::new(&vec![1; 20]), vec![], "Commit_1");
        save_object(&object, &object.id).unwrap();
        let _ = load_object::<Commit>(&object.id).unwrap();
    }

    #[test]
    fn test_format_and_parse_commit_msg() {
        {
            let msg = "commit message";
            let gpg_sig =
                "gpgsig -----BEGIN PGP SIGNATURE-----\ncontent\n-----END PGP SIGNATURE-----";
            let msg_gpg = format_commit_msg(msg, Some(gpg_sig));
            let (msg_, gpg_sig_) = parse_commit_msg(&msg_gpg);
            assert_eq!(msg, msg_);
            assert_eq!(gpg_sig, gpg_sig_.unwrap());

            let msg_gpg = format_commit_msg(msg, None);
            let (msg_, gpg_sig_) = parse_commit_msg(&msg_gpg);
            assert_eq!(msg, msg_);
            assert_eq!(None, gpg_sig_);
        }

        {
            let msg = "commit message";
            let gpg_sig =
                "gpgsig -----BEGIN PGP SIGNATURE-----\ncontent\n-----END PGP SIGNATURE-----\n \n \n";
            let msg_gpg = format_commit_msg(msg, Some(gpg_sig));
            let (msg_, _) = parse_commit_msg(&msg_gpg);
            assert_eq!(msg, msg_);
        }
    }
}
