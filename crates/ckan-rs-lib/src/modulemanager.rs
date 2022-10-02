use crate::metadb::ckan;

pub mod dependencyresolver;

enum InstallReason {
	AsDep,
	Explicit,
}

pub struct InstalledModule {
	identifier: String,
	version: ckan::ModVersion,
	reason: InstallReason,
}