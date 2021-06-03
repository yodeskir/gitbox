use git2::build::CheckoutBuilder;
use git2::build::RepoBuilder;
use git2::Error;
use git2::FetchOptions;
use git2::RemoteCallbacks;
use git2::Repository;
use std::path::Path;

pub fn do_clone<'a>(url: &str, path: &str) -> Result<Repository, Error> {
	let cb = RemoteCallbacks::new();
	let co = CheckoutBuilder::new();
	let mut fo = FetchOptions::new();
	fo.remote_callbacks(cb);
	let repo = match RepoBuilder::new()
		.fetch_options(fo)
		.with_checkout(co)
		.clone(url, Path::new(path))
	{
		Ok(repo) => repo,
		Err(e) => panic!("failed to clone: {}", e),
	};

	Ok(repo)
}
