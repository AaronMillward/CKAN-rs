use crate::metadb::ckan;

pub mod depenecyresolver;

#[derive(Debug)]
enum InstallReason {
	AsDep,
	Explicit,
}

#[derive(Debug)]
pub struct InstalledModule {
	identifier: String,
	version: ckan::ModVersion,
	reason: InstallReason,
}