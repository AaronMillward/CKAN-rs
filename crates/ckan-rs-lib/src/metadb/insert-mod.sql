INSERT INTO "mod" (
	spec_version,
	name,
	abstract,
	download_url,
	version,
	description,
	release_status,
	ksp_version,
	ksp_version_min,
	ksp_version_max,
	ksp_version_strict,
	install,
	download_size,
	download_hash_sha1,
	download_hash_sha256,
	download_content_type,
	release_date,
	resources,
	depends,
	recommends,
	suggests,
	supports,
	conflicts,
	replaced_by,
	identifier_id
)
VALUES (
	?1 ,?2 ,?3 ,?4 ,?5 ,?6 ,?7 ,?8 ,?9 ,?10,?11,?12,?13,?14,?15,?16,
	?17,?18,?19,?20,?21,?22,?23,?24,?25
)