use std::collections::HashSet;

pub enum MutlipleProvidersDecisionValidation {
	Valid(MutlipleProvidersDecisionFinal),
	Invalid(MutlipleProvidersDecision),
}

/// Describes a decision to be made by the user when mutiple providers are available for a given module
#[derive(Debug)]
pub struct MutlipleProvidersDecision {
	/* TODO: reason field */
	options: HashSet<String>,
}

impl MutlipleProvidersDecision {
	pub fn new(options: HashSet<String>) -> Self {
		MutlipleProvidersDecision { options }
	}

	pub fn get_options(&self) -> &HashSet<String> {
		&self.options
	}

	pub fn select(self, choice: String) -> MutlipleProvidersDecisionValidation {
		if self.options.contains(&choice) {
			MutlipleProvidersDecisionValidation::Valid(MutlipleProvidersDecisionFinal { selection: choice })
		} else {
			MutlipleProvidersDecisionValidation::Invalid(self)
		}
	}
}

pub struct MutlipleProvidersDecisionFinal {
	pub selection: String,
}