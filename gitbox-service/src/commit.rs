use git2::Cred;
use git2::PushOptions;
use git2::RemoteCallbacks;
use git2::{Commit, IndexAddOption, ObjectType, Oid, Repository, Signature};
use std::io::{self, Write};
use std::env;


pub fn do_fetch<'a>(
	repo: &'a git2::Repository,
	refs: &[&str],
	remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {
	let mut cb = git2::RemoteCallbacks::new();

	// Print out our transfer progress.
	cb.transfer_progress(|stats| {
		if stats.received_objects() == stats.total_objects() {
			print!(
				"Resolving deltas {}/{}\r",
				stats.indexed_deltas(),
				stats.total_deltas()
			);
		} else if stats.total_objects() > 0 {
			print!(
				"Received {}/{} objects ({}) in {} bytes\r",
				stats.received_objects(),
				stats.total_objects(),
				stats.indexed_objects(),
				stats.received_bytes()
			);
		}
		io::stdout().flush().unwrap();
		true
	});

	let mut fo = git2::FetchOptions::new();
	fo.remote_callbacks(cb);
	// Always fetch all tags.
	// Perform a download and also update tips
	fo.download_tags(git2::AutotagOption::All);
	println!("Fetching {} for repo", remote.name().unwrap());
	remote.fetch(refs, Some(&mut fo), None)?;

	// If there are local objects (we got a thin pack), then tell the user
	// how many objects we saved from having to cross the network.
	let stats = remote.stats();
	if stats.local_objects() > 0 {
		println!(
			"\rReceived {}/{} objects in {} bytes (used {} local \
             objects)",
			stats.indexed_objects(),
			stats.total_objects(),
			stats.received_bytes(),
			stats.local_objects()
		);
	} else {
		println!(
			"\rReceived {}/{} objects in {} bytes",
			stats.indexed_objects(),
			stats.total_objects(),
			stats.received_bytes()
		);
	}

	let fetch_head = repo.find_reference("FETCH_HEAD")?;
	Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
	let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
	obj
		.into_commit()
		.map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

pub fn add_and_commit(repo: &Repository, message: &str) -> Result<Oid, git2::Error> {
	let mut index = repo.index()?;
	index
		.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
		.expect("failed to add path");
	index.write()?;
	let tree_id = index.write_tree()?;
	let tree = repo.find_tree(tree_id)?;
	let signature = Signature::now(&whoami::realname(), &whoami::username())?;
	let parent_commit = find_last_commit(&repo)?;

	repo.commit(
		Some("HEAD"), //  point HEAD to our new commit
		&signature,   // author
		&signature,   // committer
		message,      // commit message
		&tree,        // tree
		&[&parent_commit],
	) // parents
}

pub fn push(repo: &Repository, url: &str, branch: &str) -> Result<(), git2::Error> {
	println!("Pushing to: {}", url);
	let mut remote = match repo.find_remote("origin") {
		Ok(r) => r,
		Err(_) => repo.remote("origin", url)?,
	};
	let mut push_options = PushOptions::default();
	let callbacks = create_callbacks();
	push_options.remote_callbacks(callbacks);
	let refs = format!("refs/heads/{}:refs/heads/{}", branch, branch);
	remote.push(
		&[refs],
		Some(&mut push_options),
	)
}

fn create_callbacks<'a>() -> RemoteCallbacks<'a> {
	let user_name = env::var("__GITHUB_USERNAME").unwrap();
	let pass = env::var("__GITHUB_PASSWORD").unwrap();

	let mut callbacks = RemoteCallbacks::new();
	&callbacks
		.credentials(move |_str, _str_opt, _cred_type| Cred::userpass_plaintext(&user_name, &pass));
	callbacks
}
