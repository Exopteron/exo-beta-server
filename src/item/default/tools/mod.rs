pub mod hoe;
pub mod pickaxe;
pub mod axe;
pub mod sword;
pub mod shovel;
#[derive(Clone, Copy, PartialEq)]
pub enum ToolMaterials {
    Wood,
    Stone,
    Iron,
}
impl PartialOrd for ToolMaterials {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.harvest_level().partial_cmp(&other.harvest_level())       
    }
}
impl ToolMaterials {
    pub fn harvest_level(&self) -> usize {
        match self {
            ToolMaterials::Wood => 0,
            ToolMaterials::Stone => 1,
            ToolMaterials::Iron => 2,
        }
    }
    pub fn max_uses(&self) -> i16 {
        match self {
            ToolMaterials::Wood => 59,
            ToolMaterials::Stone => 131,
            ToolMaterials::Iron => 250,
        }
    }
}