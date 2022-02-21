use std::{ops::Deref, sync::Arc};

use arrayvec::ArrayVec;
use j4rs::{ClasspathEntry, Jvm, JvmBuilder};
use serde::{Deserialize, Serialize};

use crate::item::{
    crafting::{normalize, print_grid, Grid, ShapedRecipe, ShapelessRecipe},
    item::ItemRegistry,
    stack::ItemStack,
};

pub struct JVMSetup;

impl JVMSetup {
    pub fn setup(mc_jar: &str) -> anyhow::Result<()> {
        let jvm = JvmBuilder::new()
            .classpath_entry(ClasspathEntry::new(mc_jar))
            .build()?;
        jvm.invoke_static("com.exopteron.RustInterface", "init", &[])?;
/*         let dump = jvm.invoke_static("com.exopteron.RustInterface", "dumpRecipes", &[])?;
        let dump: RecipeDump = jvm.to_rust(dump)?;
        let string = serde_json::to_string_pretty(&dump)?;
        std::fs::write("local/recipe_dump_jvm.json", string)?;
        let dump: RustRecipeDump = dump.into();
        let string = serde_json::to_string_pretty(&dump)?;
        std::fs::write("local/recipe_dump.json", string)?;
        log::info!(
            "{} shaped, {} shapeless",
            dump.shaped.len(),
            dump.shapeless.len()
        );
        let mut solver = (&(*ItemRegistry::global().deref()).clone()).clone();
        for recipe in dump.shaped {
            solver.solver.register_shaped(recipe);
        }
        for recipe in dump.shapeless {
            solver.solver.register_shapeless(recipe);
        }
        solver.set(); */
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ShapedRecipeJVM {
    recipe_width: i32,
    recipe_height: i32,
    recipe_items: Vec<ItemStack>,
    recipe_output: ItemStack,
}

#[derive(Serialize, Deserialize)]
pub struct ShapelessRecipeJVM {
    recipe_output: ItemStack,
    recipe_items: Vec<ItemStack>,
}

#[derive(Serialize, Deserialize)]
pub struct RecipeDump {
    shaped: Vec<ShapedRecipeJVM>,
    shapeless: Vec<ShapelessRecipeJVM>,
}
#[derive(Serialize, Deserialize)]
pub struct RustRecipeDump {
    shaped: Vec<ShapedRecipe>,
    shapeless: Vec<ShapelessRecipe>,
}
impl From<RecipeDump> for RustRecipeDump {
    fn from(other: RecipeDump) -> Self {
        let mut shaped = Vec::new();
        let mut shapeless = Vec::new();
        for recipe in other.shaped {
            if recipe.recipe_output.id() != 0 && recipe.recipe_items.len() > 0 {
                shaped.append(&mut recipe.into());
            }
        }
        for recipe in other.shapeless {
            if recipe.recipe_output.id() != 0 && recipe.recipe_items.len() > 0 {
                shapeless.push(recipe.into());
            }
        }
        RustRecipeDump { shaped, shapeless }
    }
}
impl From<ShapedRecipeJVM> for Vec<ShapedRecipe> {
    fn from(other: ShapedRecipeJVM) -> Self {
        let mut exit = other.recipe_output.id() == 96;
        log::info!("\n\n\n\n\n\n\n\n");
        let mut vec = Vec::new();
        let mut grid = Grid::default();
        log::info!("Other: {}", other.recipe_items.len());
        let flag = other.recipe_width > other.recipe_height;
        let items: Vec<&[ItemStack]> = other.recipe_items.chunks(3).collect();
        let mut x = other.recipe_width as usize;
        let mut y = other.recipe_height as usize;
        if false {
            x = other.recipe_height as usize;
            y = other.recipe_width as usize;
        }
        for x in 0..3 {
            for y in 0..3 {
                grid[y][x] = convert(other.recipe_items[x + y * other.recipe_width as usize].clone());
            }
        }
        normalize(&mut grid);
        print_grid(&grid);
        vec.push(        ShapedRecipe {
            input: grid,
            output: other.recipe_output.clone(),
        });

        let mut grid = Grid::default();
        log::info!("Other: {}", other.recipe_items.len());
        let flag = other.recipe_width > other.recipe_height;
        let items: Vec<&[ItemStack]> = other.recipe_items.chunks(3).collect();
        let mut x = other.recipe_height as usize;
        let mut y = other.recipe_width as usize;
        if flag {
            x = other.recipe_width as usize;
            y = other.recipe_height as usize;
        }
        for x in 0..x {
            for y in 0..y {
                grid[y][x] = convert(other.recipe_items[other.recipe_width as usize - x - 1 + y * other.recipe_width as usize].clone());
            }
        }
        normalize(&mut grid);
        print_grid(&grid);
        vec.push(        ShapedRecipe {
            input: grid,
            output: other.recipe_output,
        });
        if exit {
            //std::process::exit(0);
        }
        vec
    }
}
impl From<ShapelessRecipeJVM> for ShapelessRecipe {
    fn from(other: ShapelessRecipeJVM) -> Self {
        let mut vec = ArrayVec::new();
        for item in &other.recipe_items {
            if let Some(item) = convert(item.clone()) {
                vec.push(item);
            }
        }
        ShapelessRecipe {
            input: vec,
            output: other.recipe_output,
        }
    }
}
fn convert(mut input: ItemStack) -> Option<ItemStack> {
    if input.null_flag || input.id() == 0 {
        None
    } else {
        input.set_damage(0);
        Some(input)
    }
}
