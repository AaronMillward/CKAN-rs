use super::*;
use package::*;
use crate::Error::Parse;

/// Gets the lastest MetaDB .tar.gz archive as bytes
async fn get_latest_archive() -> crate::Result<Vec<u8>> {
	/* TODO: Latest archive URL not hardcoded instead in CkanRsConfig */
	log::trace!("Downloading latest MetaDB.");
	Ok(reqwest::get("https://github.com/KSP-CKAN/CKAN-meta/archive/master.tar.gz").await?.bytes().await.map(|v| v.to_vec())?)
}

/// Download and generate the latest MetaDB.
pub async fn generate_latest() -> crate::Result<MetaDB> {
	log::trace!("Generating latest MetaDB.");
	let archive_data = get_latest_archive().await?;
	let mut gz = flate2::bufread::GzDecoder::new(archive_data.as_slice());
	let mut v = Vec::<u8>::new();
	gz.read_to_end(&mut v)?;
	MetaDB::generate_from_archive(
		&mut tar::Archive::new(v.as_slice()), 
		true
	)
}

impl MetaDB {
	/// Creates a new MetaDB using a tar archive.
	/// # Parameters
	/// - `archive` - A tarball containing the metadb json files, should *not* be compressed.
	/// - `do_validation` - Usually enabled when the repo can't be trusted to validate their ckans. should be `false` for most cases as it is slow.
	pub fn generate_from_archive<R>(archive: &mut tar::Archive<R>, do_validation: bool) -> crate::Result<Self>
	where R: std::io::Read
	{
		log::trace!("Generating MetaDB from given archive.");
		/* TODO: Determine if this is IO or CPU bound causing it to take 15 sec to generate. */
		/* TODO: Some entries fail when validated against the schema, should this happen? surely the remote repo
		doesn't have incorrect entries? */

		let mut packages = HashSet::<Package>::new();
		let mut builds = HashMap::<i32, String>::new();

		let compiled_schema = do_validation.then(||
			jsonschema::JSONSchema::compile(
				&serde_json::from_str(
					include_str!("CKAN-json.schema")
				).expect("schema should be valid json.")
			).expect("schema should compile.")
		);

		for (i, entry) in archive.entries()?.enumerate() {
			let mut entry = entry.map_err(|_| Parse("tar archive entries unreadable".to_string()))?;

			if entry.header().entry_type() == tar::EntryType::Directory {
				continue;
			}

			if entry.path()?.to_string_lossy() == "CKAN-meta-master/builds.json" {
				log::trace!("Processing builds.json");
				let mut buffer = Vec::<u8>::new();
				entry.read_to_end(&mut buffer)?;
				let json: serde_json::Value = serde_json::from_str(&String::from_utf8(buffer).unwrap())?;
				builds = serde_json::from_value(
					json.as_object().expect("builds.json root should be an object.")
					.get("builds").expect("builds.json root object should contain key \"builds\".")
					.clone()
				)?;
				continue;
			}

			let json = {
				let mut buffer = Vec::<u8>::new();
				entry.read_to_end(&mut buffer)?;
				match serde_json::from_slice::<serde_json::Value>(&buffer) {
					Ok(v) => v,
					Err(e) => {
						log::warn!("Couldn't process entry {} in metadb archive, failed to deserialize as JSON: {}", i, e);
						continue;
					},
				}
			};
			
			if let Some(schema) = &compiled_schema {
				if !schema.is_valid(&json) {
					log::warn!("Couldn't process entry {} in metadb, it does not match the schema", i);
					continue;
				}
			}

			{
				let ckan : Package = match Package::read_from_json(json) {
					Ok(v) => v,
					Err(e) => {
						log::warn!("Couldn't process entry {} in metadb, failed to create package from JSON: {}", i, e);
						continue;
					},
				};
				packages.insert(ckan);
			}
		}

		assert!(!builds.is_empty(), "builds.json was not processed.");

		Ok(Self { packages, builds })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ckan_json_schema_compiles() {
		jsonschema::JSONSchema::compile(
			&serde_json::from_str(
				include_str!("CKAN-json.schema")
			).expect("schema isn't valid json")
		).expect("schema isn't invalid");
	}

	#[tokio::test]
	async fn get_lastest_db_archive() {
		let a = get_latest_archive().await.expect("failed to download archive.");
		if a.is_empty() {
			panic!("data is empty.")
		}
		if a.len() < 2 * 1000 * 1000 { /* The repo as of 2022-09-20 totals roughly 3mb in .tar.gz form so 2mb seems like a sensible value */
			panic!("data seems too small <2mb.")
		}
	}
}