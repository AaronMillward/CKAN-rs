use std::collections::HashMap;

use super::ckan::*;

pub struct KspVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Ckan>,
{
	match_versions: Vec<KspVersion>,
	underlying: I,
}

impl<'a, I> Iterator for KspVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Ckan>,
{
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		for module in self.underlying.by_ref() {
			/* TODO: ksp_version_strict | this needs to be fixed in KspVersion */

			let does_match = match &module.ksp_version {
				VersionBounds::Any => true,
				VersionBounds::Explicit(v) => self.match_versions.iter().any(|ksp| KspVersion::is_sub_version(ksp, v)),
				VersionBounds::MinOnly(min) => self.match_versions.iter().any(|ksp| ksp <= min),
				VersionBounds::MaxOnly(max) => self.match_versions.iter().any(|ksp| ksp <= max),
				VersionBounds::MinMax(min, max) => self.match_versions.iter().any(|ksp| min <= ksp && ksp <= max),
			};

			if does_match {
				return Some(module)
			} else {
				continue;
			}
		}
		None
	}
}

pub trait KspVersionMatchesExt<'a>: Iterator<Item = &'a Ckan>
{
	/// Filters the iterator to modules compatible with `match_versions`
	fn ksp_version_matches(self, match_versions: Vec<KspVersion>) -> KspVersionMatches<'a, Self>
	where
		Self: Sized,
	{
		KspVersionMatches { underlying: self, match_versions}
	}
}

impl<'a, I: Iterator<Item = &'a Ckan>> KspVersionMatchesExt<'a> for I {}


pub struct ModVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Ckan>,
{
	bounds: ModVersionBounds,
	underlying: I,
}

impl<'a, I> Iterator for ModVersionMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Ckan>,
{
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		for module in self.underlying.by_ref() {
			if self.bounds.is_version_within(&module.unique_id.version) {
				return Some(module)
			} else {
				continue;
			}
		}
		None
	}
}

pub trait ModVersionMatchesExt<'a>: Iterator<Item = &'a Ckan>
{
	/// Filters the iterator to modules matching the requirements of `bounds`
	fn mod_version_matches(self, bounds: ModVersionBounds) -> ModVersionMatches<'a, Self>
	where
		Self: Sized,
	{
		ModVersionMatches { underlying: self, bounds}
	}
}

impl<'a, I: Iterator<Item = &'a Ckan>> ModVersionMatchesExt<'a> for I {}


pub struct DescriptorMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Ckan>,
{
	descriptor: ModuleDescriptor,
	underlying: I,
}

impl<'a, I> Iterator for DescriptorMatches<'a, I>
where
	I: Iterator<Item = &'a super::ckan::Ckan>,
{
	type Item = I::Item;

	fn next(&mut self) -> Option<Self::Item> {
		for module in self.underlying.by_ref() {
			if does_module_provide_descriptor(module, &self.descriptor) {
				return Some(module)
			}
		}
		None
	}
}

pub trait DescriptorMatchesExt<'a>: Iterator<Item = &'a Ckan>
{
	/// Filters the iterator to only modules matching the descriptor including `provides` relationships.
	/// This means the output may not be all the same identifier.
	fn descriptor_matches(self, descriptor: ModuleDescriptor) -> DescriptorMatches<'a, Self>
	where
		Self: Sized,
	{
		DescriptorMatches { underlying: self, descriptor}
	}
}

impl<'a, I: Iterator<Item = &'a Ckan>> DescriptorMatchesExt<'a> for I {}

pub trait GetProvidersExt<'a>: Iterator<Item = &'a Ckan>
{
	/// Filters the iterator to modules compatible with `match_versions`
	fn get_modules_providing(mut self, descriptor: &ModuleDescriptor) -> HashMap<String, Vec<&'a Ckan>>
	where
		Self: Sized,
	{
		let mut map = HashMap::<String, Vec<&'a Ckan>>::new();
		for module in self.by_ref() {
			if does_module_provide_descriptor(module, descriptor) {
				map.entry(module.unique_id.identifier.clone()).or_default().push(module);
			}
		}
		map
	}
}

impl<'a, I: Iterator<Item = &'a Ckan>> GetProvidersExt<'a> for I {}

/* TODO: Tests */