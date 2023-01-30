use std::collections::HashMap;

use super::ckan::*;

pub struct KspVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Package>,
{
	match_versions: Vec<KspVersion>,
	underlying: I,
}

impl<'a, I> Iterator for KspVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Package>,
{
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		for package in self.underlying.by_ref() {
			/* TODO: ksp_version_strict | this needs to be fixed in KspVersion */

			let does_match = match &package.ksp_version {
				VersionBounds::Any => true,
				VersionBounds::Explicit(v) => self.match_versions.iter().any(|ksp| KspVersion::is_sub_version(ksp, v)),
				VersionBounds::MinOnly(min) => self.match_versions.iter().any(|ksp| ksp <= min),
				VersionBounds::MaxOnly(max) => self.match_versions.iter().any(|ksp| ksp <= max),
				VersionBounds::MinMax(min, max) => self.match_versions.iter().any(|ksp| min <= ksp && ksp <= max),
			};

			if does_match {
				return Some(package)
			} else {
				continue;
			}
		}
		None
	}
}

pub trait KspVersionMatchesExt<'a>: Iterator<Item = &'a Package>
{
	/// Filters the iterator to packages compatible with `match_versions`
	fn ksp_version_matches(self, match_versions: Vec<KspVersion>) -> KspVersionMatches<'a, Self>
	where
		Self: Sized,
	{
		KspVersionMatches { underlying: self, match_versions}
	}
}

impl<'a, I: Iterator<Item = &'a Package>> KspVersionMatchesExt<'a> for I {}


pub struct ModVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Package>,
{
	bounds: PackageVersionBounds,
	underlying: I,
}

impl<'a, I> Iterator for ModVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Package>,
{
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		for package in self.underlying.by_ref() {
			if self.bounds.is_version_within(&package.identifier.version) {
				return Some(package)
			} else {
				continue;
			}
		}
		None
	}
}

pub trait ModVersionMatchesExt<'a>: Iterator<Item = &'a Package>
{
	/// Filters the iterator to packages matching the requirements of `bounds`
	fn mod_version_matches(self, bounds: PackageVersionBounds) -> ModVersionMatches<'a, Self>
	where
		Self: Sized,
	{
		ModVersionMatches { underlying: self, bounds}
	}
}

impl<'a, I: Iterator<Item = &'a Package>> ModVersionMatchesExt<'a> for I {}


pub struct DescriptorMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Package>,
{
	descriptor: PackageDescriptor,
	underlying: I,
}

impl<'a, I> Iterator for DescriptorMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Package>,
{
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		for package in self.underlying.by_ref() {
			if does_package_provide_descriptor(package, &self.descriptor) {
				return Some(package)
			}
		}
		None
	}
}

pub trait DescriptorMatchesExt<'a>: Iterator<Item = &'a Package>
{
	/// Filters the iterator to only packages matching the descriptor including `provides` relationships.
	/// This means the output may not be all the same identifier.
	fn descriptor_matches(self, descriptor: PackageDescriptor) -> DescriptorMatches<'a, Self>
	where
		Self: Sized,
	{
		DescriptorMatches { underlying: self, descriptor}
	}
}

impl<'a, I: Iterator<Item = &'a Package>> DescriptorMatchesExt<'a> for I {}

pub trait GetProvidersExt<'a>: Iterator<Item = &'a Package>
{
	/// Filters the iterator to packages compatible with `match_versions`
	fn get_packages_providing(mut self, descriptor: &PackageDescriptor) -> HashMap<String, Vec<&'a Package>>
	where
		Self: Sized,
	{
		let mut map = HashMap::<String, Vec<&'a Package>>::new();
		for package in self.by_ref() {
			if does_package_provide_descriptor(package, descriptor) {
				map.entry(package.identifier.identifier.clone()).or_default().push(package);
			}
		}
		map
	}
}

impl<'a, I: Iterator<Item = &'a Package>> GetProvidersExt<'a> for I {}

/* TODO: Tests */