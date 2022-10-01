/* This DB is only written once when it's first created
 * and is regenerated consistently so there isn't
 * much need for updates, etc
 */

CREATE TABLE "mod" (
	"id"                    INTEGER         ,
	"spec_version"          TEXT    NOT NULL,
	"name"                  TEXT    NOT NULL,
	"abstract"              TEXT    NOT NULL,
	"download_url"          TEXT            ,
	"version"               BLOB    NOT NULL,
	"description"           TEXT            ,
	"release_status"        INTEGER         ,
	"ksp_version"           TEXT            ,
	"ksp_version_min"       TEXT            ,
	"ksp_version_max"       TEXT            ,
	"ksp_version_strict"    BOOLEAN         ,
	"install"               BLOB            ,
	"download_size"         INTEGER         ,
	"download_hash_sha1"    BLOB            ,
	"download_hash_sha256"  BLOB            ,
	"download_content_type" TEXT            ,
	"release_date"          TEXT            ,
	"resources"             BLOB            ,
	
	/* Relationships */
	"depends"               BLOB            ,
	"recommends"            BLOB            ,
	"suggests"              BLOB            ,
	"supports"              BLOB            ,
	"conflicts"             BLOB            ,
	"replaced_by"           BLOB            ,

	PRIMARY KEY("id" AUTOINCREMENT),
);

CREATE TABLE "author" (
	"id"   INTEGER         ,
	"name" TEXT    NOT NULL,
	PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE "mod_author" (
	"mod_id"    INTEGER NOT NULL,
	"author_id" INTEGER NOT NULL,
	FOREIGN KEY("author_id") REFERENCES "author"("id") ON UPDATE CASCADE ON DELETE RESTRICT,
	FOREIGN KEY("mod_id")    REFERENCES "mod"("id")    ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE "mod_license" (
	"mod_id" INTEGER NOT NULL,
	"type"   TEXT    NOT NULL,
	FOREIGN KEY("mod_id") REFERENCES "mod"("id") ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE "mod_tag" (
	"mod_id" INTEGER NOT NULL,
	"name"   TEXT    NOT NULL,
	FOREIGN KEY("mod_id") REFERENCES "mod"("id") ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE "mod_localization" (
	"mod_id"   INTEGER NOT NULL,
	"language" TEXT    NOT NULL,
	FOREIGN KEY("mod_id") REFERENCES "mod"("id") ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE "identifier" (
	"id"             INTEGER         ,
	"name"           TEXT    NOT NULL,
	"download_count" INTEGER         ,
	"virtual"        BOOLEAN         , --Indicates if this identifier represents a single mod or multiple
	PRIMARY KEY("id" AUTOINCREMENT)
);

/* Describes what identifers a given mod provides */
CREATE TABLE "mod_provides" (
	"mod_id" INTEGER,
	"identifier_id" INTEGER,
)