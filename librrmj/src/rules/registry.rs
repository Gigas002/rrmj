use crate::rules::profile_trait::RulesProfile;
use crate::rules::RulesProfileId;
use crate::rules::standard::StandardRules;
use crate::Error;

pub struct RulesRegistry;

impl RulesRegistry {
    pub fn get(id: RulesProfileId) -> Result<&'static dyn RulesProfile, Error> {
        match id {
            RulesProfileId::Standard => Ok(&StandardRules),
        }
    }
}
