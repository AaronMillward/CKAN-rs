//! Handles installing modules to a game directory

/* 
I quickly searched through the metadb using vim and it seems like the *vast* majority of content types are `application/zip`
so we're just going to consider them all zips and error otherwise for now.
 */

/* TODO: A potential staging step between extraction and deployment to allow for file merges and other overwrites. */

pub mod download;
pub mod content;
pub mod deployment;