use crate::game::DamageType;

// feather license in FEATHER_LICENSE.md

/// Represents an entity's health
#[derive(
    Clone, Debug, PartialEq,
)]
pub struct Health(pub i16, pub DamageType);
impl Health {
    pub fn damage(&mut self, amount: i16, damage_type: DamageType) {
        self.0 -= amount;
        self.1 = damage_type;
    }
}
pub struct Dead;
pub struct PreviousHealth(pub Health);
/// Represents an entity's hunger
#[derive(
    Copy, Clone, Debug, PartialEq,
)]
pub struct Hunger(pub i16, pub f32);
