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
        rules.insert("random-tick-speed".to_string(), GameruleValue::Int(3));
        Self { rules: rules }
    }
}
impl Gamerules {
    pub fn get_int(&self, name: &str) -> i32 {
        if let GameruleValue::Int(val) = self.rules.get(name).expect("Rule does not exist!") {
            return *val;
        }
        panic!("Wrong value!");
    }
    pub fn get_boolean(&self, name: &str) -> bool {
        if let GameruleValue::Boolean(val) = self.rules.get(name).expect("Rule does not exist!") {
            return *val;
        }
        panic!("Wrong value!");
    }

}