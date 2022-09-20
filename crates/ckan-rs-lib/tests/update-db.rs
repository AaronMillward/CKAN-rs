#[test]
fn metadb_get_latest_archive() {
	let a = ckan_rs::metadb::get_latest_archive().expect("failed to download archive");
	if a.is_empty() {
		panic!("data is empty")
	}
	if a.len() < 2 * 1000 * 1000 { /* The repo as of 2022-09-20 totals roughly 3mb in .tar.gz form so 2mb seems like a sensible value */
		panic!("data seems too small <2mb")
	}
}