use super::*;
pub enum GameruleValue {
    Boolean(bool),
    String(String),
    Int(i32),
}
pub struct Gamerules {
    pub rules: HashMap<String, GameruleValue>    
}
impl std::default::Default for Gamerules {
    fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert("pvp-enabled".to_string(), GameruleValue::Boolean(true));
        rules.insert("fall-damage".to_string(), GameruleValue::Boolean(true));
        Self { rules: rules }
    }
}