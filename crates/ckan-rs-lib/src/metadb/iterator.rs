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
			if let Some(version) = &module.ksp_version {
				if version.is_any() ||
					self.match_versions.iter().any(|ksp| KspVersion::is_sub_version(ksp, version))
				{
					return Some(module)
				} else {
					continue;
				}
			}
			let b = match (&module.ksp_version_min, &module.ksp_version_max) {
				/* XXX: at this point we can deduce the module has no ksp version requirements. The spec says this means it is "any" */
				(None     , None)      => { true },
				(None     , Some(max)) => { self.match_versions.iter().any(|ksp| ksp <= max) },
				(Some(min), None)      => { self.match_versions.iter().any(|ksp| ksp >= min) },
				(Some(min), Some(max)) => { self.match_versions.iter().any(|ksp| min <= ksp && ksp <= max) }
			};
			if b {
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
			if does_module_match_descriptor(module, &self.descriptor) {
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