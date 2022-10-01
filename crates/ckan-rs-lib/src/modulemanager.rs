mod depenecyresolver;

enum InstallReason {
	AsDep,
	Explicit,
}

#[derive(Debug, Default)]
pub struct InstalledModule {
	identifier: String,
	version: ModVersion,
	reason: InstallReason,
}