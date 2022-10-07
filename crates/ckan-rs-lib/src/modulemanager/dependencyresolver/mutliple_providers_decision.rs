use std::collections::HashSet;

/// Describes a decision to be made by the user when mutiple providers are available for a given module
#[derive(Debug)]
pub struct MutlipleProvidersDecision {
	/* TODO: reason field */
	options: HashSet<String>,
	selection: String,
}

impl MutlipleProvidersDecision {
	pub fn new(options: HashSet<String>) -> Self {
		MutlipleProvidersDecision {
			options,
			selection: "".to_string(),
		}
	}

	pub fn get_options(&self) -> &HashSet<String> {
		&self.options
	}

	pub fn select(&mut self, choice: String) -> bool {
		if !self.options.contains(&choice) {
			false
		} else {
			self.selection = choice;
			true
		}
	}

	pub fn get_decision(self) -> String {
		self.selection
	}

	/* TODO: Some `finalize` function to stop users from passing incomplete decisions back to the resolver */
}