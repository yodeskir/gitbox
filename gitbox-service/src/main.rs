extern crate confy;
extern crate notify;
#[macro_use]
extern crate serde_derive;

mod commit;
mod gitinit;

use crate::commit::add_and_commit;
use crate::commit::do_fetch;
use crate::commit::push;
use daemon::Daemon;
use daemon::DaemonRunner;
use daemon::State;
use git2::Repository;
use notify::{watcher, DebouncedEvent::*, RecursiveMode, Watcher};
use pathdiff::diff_paths;
use std::env;
use std::io::Error;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct GitBoxConfig {
	repourl: String,
	branch: String,
	localwatch: String,
	username: String,
	password: String,
	cloned: bool,
}

impl Default for GitBoxConfig {
	fn default() -> Self {
		GitBoxConfig {
			repourl: "".to_string(),
			branch: "".to_string(),
			localwatch: "".to_string(),
			username: "".to_string(),
			password: "".to_string(),
			cloned: false,
		}
	}
}

fn main() -> Result<(), Error> {
	let daemon = Daemon {
		name: "gitbox-service".to_string(),
	};
	daemon.run(move |rx: Receiver<State>| {
		for signal in rx.iter() {
			match signal {
				State::Start => start(),
				State::Reload => start(),
				State::Stop => stop(),
			};
		}
	})?;
	Ok(())
}

fn stop() {
	std::process::exit(0x0100);
}

fn start() -> () {
	let cfg: GitBoxConfig = confy::load("gitbox-service").expect("Failed to read config file");
	clone_if_new(&cfg);

	let local: String = String::from(&cfg.localwatch);
	let repo_url: String = String::from(&cfg.repourl);
	let branch_name: String = String::from(&cfg.branch);
	let username: String = String::from(&cfg.username);
	let password: String = String::from(&cfg.password);
	env::set_var("__GITHUB_USERNAME", username);
	env::set_var("__GITHUB_PASSWORD", password);

	let (tx, rx) = channel();
	let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();
	watcher.watch(&local, RecursiveMode::Recursive).unwrap();
	loop {
		match rx.recv() {
			Ok(event) => match &event {
				Rename(src, dest) => {
					let relative_path = diff_paths(src, &local).unwrap();
					let msg = format!("Renaming {:?} to {:?}", &relative_path, &dest);
					commit_and_push(&repo_url, &branch_name, src.to_path_buf(), &local, msg);
				}
				Create(path) => {
					let relative_path = diff_paths(path, &local).unwrap();
					let msg = format!("Creating new {:?}", &relative_path);
					commit_and_push(&repo_url, &branch_name, path.to_path_buf(), &local, msg);
				}
				Write(path) => {
					let relative_path = diff_paths(path, &local).unwrap();
					let msg = format!("Updating {:?}", &relative_path);
					commit_and_push(&repo_url, &branch_name, path.to_path_buf(), &local, msg);
				}
				Remove(path) => {
					let relative_path = diff_paths(path, &local).unwrap();
					let msg = format!("Deleting new {:?}", &relative_path);
					commit_and_push(&repo_url, &branch_name, path.to_path_buf(), &local, msg);
				}
				_ => (),
			},
			Err(_e) => break,
		};
	}
}

fn fetch(dir: &str, branch: &str) {
	let repo = Repository::open(dir).expect("Couldn't open repository");
	let remote_name = "origin";
	let remote_branch = branch;
	let mut remote = repo
		.find_remote(remote_name)
		.expect("Failed to find remote");
	let _ = do_fetch(&repo, &[remote_branch], &mut remote);
}

fn commit_and_push(repo_url: &str, branch: &str, src: PathBuf, dir: &str, message: String) -> () {
	let repo = Repository::open(dir).expect("Couldn't open repository");
	let str_path = String::from(src.to_str().unwrap());
	let bad_git = ".git";
	if !str_path.contains(bad_git) {
		fetch(&dir, branch);
		let _ = add_and_commit(&repo, &message);
		let _p = push(&repo, repo_url, branch).expect("Couldn't push to remote repo");
	}
}

fn clone_if_new(cfg: &GitBoxConfig) -> () {
	if cfg.cloned {
		()
	} else {
		gitinit::do_clone(&cfg.repourl, &cfg.localwatch).expect("Failed to clone");
		let cfgupd = GitBoxConfig {
			repourl: String::from(&cfg.repourl),
			branch: String::from(&cfg.branch),
			localwatch: String::from(&cfg.localwatch),
			username: String::from(&cfg.username),
			password: String::from(&cfg.password),
			cloned: true,
		};

		confy::store("gitbox-service", cfgupd).expect("Failed to update config");
	}
}
