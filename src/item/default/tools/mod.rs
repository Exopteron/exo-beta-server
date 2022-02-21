pub mod hoe;
pub mod pickaxe;
pub mod axe;
#[derive(Clone, Copy)]
pub enum ToolMaterials {
    Wood,
    Stone
}
impl ToolMaterials {
    pub fn harvest_level(&self) -> usize {
        match self {
            ToolMaterials::Wood => 0,
            ToolMaterials::Stone => 1,
        }
    }
    pub fn max_uses(&self) -> i16 {
        match self {
            ToolMaterials::Wood => 59,
            ToolMaterials::Stone => 131
        }
    }
}