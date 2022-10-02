use std::io::Read;

#[test]
fn metadb_create() {
	let archive_data = ckan_rs::metadb::get_latest_archive().expect("failed to download archive");
	let mut gz = flate2::bufread::GzDecoder::new(archive_data.as_slice());
	let mut v = Vec::<u8>::new();
	gz.read_to_end(&mut v).unwrap();
	ckan_rs::metadb::MetaDB::generate_from_archive(
		&mut tar::Archive::new(v.as_slice()), 
		true
	).expect("failed to create db");
}