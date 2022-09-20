use super::*;

/// Gets the lastest MetaDB .tar.gz archive as bytes
pub fn get_latest_archive() -> crate::Result<Vec<u8>> {
	use std::io::Read;
	let mut v = Vec::<u8>::new();
	reqwest::blocking::get("https://github.com/KSP-CKAN/CKAN-meta/archive/master.tar.gz")?.read_to_end(&mut v)?;
	Ok(v)
}

impl MetaDB {
	/// Creates a new MetaDB at the given `path` using a tar archive.
	/// # Parameters
	/// - `path` - Where the database will be created.
	/// - `archive_data` - A tarball containing the metadb json files, should *not* be compressed.
	/// - `do_validation` - Usually enabled when the repo can't be trusted to validate their ckans. should be `false` for most cases as it is slow.
	pub fn generate_from_archive<R>(path: std::path::PathBuf, archive: &mut tar::Archive<R>, do_validation: bool) -> crate::Result<Self>
	where R: std::io::Read
	{
		use ckan::*;
		use rusqlite::params;
		use std::io::Read;
		use std::collections::HashMap;

		std::fs::remove_file(&path).ok();
		let mut conn = rusqlite::Connection::open(&path)?;
		conn.execute_batch(include_str!("metadb-schema.sql"))?; /* Create DB */
		let trans = conn.transaction()?;

		/* Loop and insert data into database */ {
			/* This can't be converted to 1 long statement due to sqlite array handling */
			let stmt_insert_mod              = &mut trans.prepare(include_str!("insert-mod.sql"))?;
			let stmt_insert_author           = &mut trans.prepare(include_str!("insert-author.sql"))?;
			let stmt_insert_mod_license      = &mut trans.prepare(include_str!("insert-mod-license.sql"))?;
			let stmt_insert_mod_author       = &mut trans.prepare(include_str!("insert-mod-author.sql"))?;
			let stmt_insert_mod_tag          = &mut trans.prepare(include_str!("insert-mod-tag.sql"))?;
			let stmt_insert_mod_localization = &mut trans.prepare(include_str!("insert-mod-localization.sql"))?;
			let stmt_insert_mod_relationship = &mut trans.prepare(include_str!("insert-mod-relationship.sql"))?;
			let stmt_insert_identifier       = &mut trans.prepare(include_str!("insert-identifier.sql"))?;

			/* Create a vector containing the read CKAN files */
			let ckans: Vec<Ckan> = {
				let mut v = Vec::<Ckan>::new();

				let compiled_schema = if do_validation {
					Some(
						jsonschema::JSONSchema::compile(&serde_json::from_str(include_str!("CKAN-json.schema")).expect("schema isn't valid json")).expect("schema isn't invalid")
					)
				} else {
					None
				};

				for (i, e) in archive.entries()?.enumerate() {
					let mut e = e.unwrap();
					if e.size() == 0 {
						continue;
					}

					let json = {
						let mut b = Vec::<u8>::new();
						e.read_to_end(&mut b)?;
						match serde_json::from_slice::<serde_json::Value>(&b) {
							Ok(v) => v,
							Err(e) => {
								eprintln!("Couldn't process entry {} in metadb, failed to deserialize as JSON: {}", i, e);
								continue;
							},
						}
					};
					
					if let Some(schema) = &compiled_schema {
						if !schema.is_valid(&json) {
							eprintln!("Couldn't process entry {} in metadb, does not match schema", i);
							continue;
						}
					}

					{
						let ckan : Ckan = match Ckan::read_from_json(json) {
							Ok(v) => v,
							Err(e) => {
								eprintln!("Couldn't process entry {} in metadb: {}", i, e);
								continue;
							},
						};
						v.push(ckan);
					}
				}
				v
			};

			/* Map of identifiers and their table id's */
			let mut identifiers: HashMap<String, i64> = HashMap::new();
			/* Map of authors and their table id's */
			let mut authors: HashMap<String, i64> = HashMap::new();

			/* Populate the identifer table */
			for c in &ckans {
				if identifiers.contains_key(&c.identifier) { continue; }
				if stmt_insert_identifier.execute(params![c.identifier])? > 0 {
					identifiers.insert(c.identifier.clone(), trans.last_insert_rowid());
				} else {
					panic!()
				}
			}

			/* Populate the author table */
			for c in &ckans {
				for author in &c.author {
					if authors.contains_key(author) { continue; }
					if stmt_insert_author.execute(params![author])? > 0 {
						authors.insert(author.clone(), trans.last_insert_rowid());
					} else {
						panic!("couldn't insert author {}", author)
					}
				}
			}

			/* Insert the mod entries */
			for c in ckans {
				let changes = stmt_insert_mod.execute(params![
					c.spec_version,
					c.name,
					c.r#abstract,
					c.download,
					c.version,
					c.description,
					c.release_status,
					c.ksp_version,
					c.ksp_version_min,
					c.ksp_version_max,
					c.ksp_version_strict,
					bincode::serialize(&c.install).ok(),
					c.download_size,
					c.download_hash_sha1,
					c.download_hash_sha256,
					c.download_content_type,
					c.release_date,
					bincode::serialize(&c.resources).ok(),
					bincode::serialize(&c.depends).ok(),
					bincode::serialize(&c.recommends).ok(),
					bincode::serialize(&c.suggests).ok(),
					bincode::serialize(&c.supports).ok(),
					bincode::serialize(&c.conflicts).ok(),
					bincode::serialize(&c.replaced_by).ok(),
					identifiers.get(&c.identifier),
				])?;
				
				if changes == 1 {
					let mod_id = trans.last_insert_rowid();

					for author in c.author {
						stmt_insert_mod_author.execute(params![mod_id, authors.get(&author).unwrap()])?; /* Same here */
					}

					if let Some(locales) = c.localizations {
						for locale in locales {
							stmt_insert_mod_localization.execute(params![mod_id, locale])?;
						}
					}

					for lic in c.license {
						stmt_insert_mod_license.execute(params![mod_id, lic])?;
					}

					if let Some(tags) = c.tags {
						for tag in tags {
							stmt_insert_mod_tag.execute(params![mod_id, tag])?;
						}
					}

					/* TODO: Relationships */
				}
			}
		}

		trans.commit()?;

		Ok(
			Self {
				connection: conn
			}
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Read;

	#[test]
	fn metadb_generate() {
		let data = include_bytes!("../../test-data/CKAN-meta-master.tar.gz");
		let mut gz = flate2::bufread::GzDecoder::new(data.as_slice());
		let mut v = Vec::<u8>::new();
		gz.read_to_end(&mut v).unwrap();
		MetaDB::generate_from_archive(std::path::PathBuf::from("/tmp/metadb.db"), &mut tar::Archive::new(v.as_slice()), true).unwrap();
	}
}