use crate::game::DamageType;
pub mod zombie;
// feather license in FEATHER_LICENSE.md


/// Represents an entity's health
#[derive(Clone, Debug, PartialEq)]
pub struct Health(pub i16, pub DamageType);
impl Health {
    pub fn damage(&mut self, amount: i16, damage_type: DamageType) {
        if self.0 > 0 {
            self.0 -= amount;
            self.1 = damage_type;
        }
    }
}
pub struct Dead;
pub struct PreviousHealth(pub Health);
/// Represents an entity's hunger
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Hunger(pub i16, pub f32);
impl Hunger {
    pub fn get_points(&mut self, num: i16) -> bool {
        if self.1 > 0. {
            self.1 -= 1.5;
            return true;
        }
        if self.0 < num {
            return false;
        }
        self.0 -= num;
        return true;
    }
}
pub struct PreviousHunger(pub Hunger);

pub struct Regenerator(pub u128);
