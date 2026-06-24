use crate::Error;
use crate::rules::RulesProfileId;
use crate::rules::profile::RulesProfile;
use crate::rules::standard::StandardRules;

pub struct RulesRegistry;

impl RulesRegistry {
    pub fn get(id: RulesProfileId) -> Result<&'static dyn RulesProfile, Error> {
        match id {
            RulesProfileId::Standard => Ok(&StandardRules),
        }
    }
}
