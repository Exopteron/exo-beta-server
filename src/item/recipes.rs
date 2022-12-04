use super::{crafting::{Solver, ShapelessRecipe, ShapedRecipe, Grid, normalize}, stack::ItemStack, furnace::FurnaceSolver};

pub fn register_furnace(recipes: &mut FurnaceSolver) {
    recipes.add_item(4, ItemStack::new(1, 1, 0));

    recipes.add_item(15, ItemStack::new(265, 1, 0));

    recipes.add_item(12, ItemStack::new(20, 1, 0));
}
pub fn register(recipes: &mut Solver) {
    let planks = ItemStack::new(5, 1, 0);
    let sticks = ItemStack::new(280, 1, 0);
    let wheat = ItemStack::new(296, 1, 0);
    let cobble = ItemStack::new(4, 1, 0);
    recipes.register_shapeless(ShapelessRecipe::new_from_vec(vec![ItemStack::new(17, 1, 0)], ItemStack::new(5, 4, 0))); // Planks
    recipes.register_shaped(ShapedRecipe::new([[Some(planks.clone()), None, None],[Some(planks.clone()), None, None],[None, None, None]], ItemStack::new(280, 4, 0)));
    recipes.register_shaped(ShapedRecipe::new([[Some(planks.clone()), Some(planks.clone()), None],[Some(planks.clone()), Some(planks.clone()), None],[None, None, None]], ItemStack::new(58, 1, 0)));
    recipes.register_shaped(ShapedRecipe::new([[Some(wheat.clone()), Some(wheat.clone()), Some(wheat.clone())],[None, None, None],[None, None, None]], ItemStack::new(297, 1, 0)));
    register_tools(recipes);
    recipes.register_shaped(ShapedRecipe::new([[Some(planks.clone()), Some(planks.clone()), Some(planks.clone())],[Some(planks.clone()), Some(planks.clone()), Some(planks.clone())],[None, Some(sticks.clone()), None]], ItemStack::new(323, 1, 0))); // sign

    recipes.register_shaped(ShapedRecipe::new([[Some(planks.clone()), Some(planks.clone()), None],[Some(planks.clone()), Some(planks.clone()), None],[Some(planks.clone()), Some(planks.clone()), None]], ItemStack::new(324, 1, 0))); // door


    recipes.register_shaped(ShapedRecipe::new([[Some(planks.clone()), Some(planks.clone()), Some(planks.clone())],[Some(planks.clone()), None, Some(planks.clone())],[Some(planks.clone()), Some(planks.clone()), Some(planks.clone())]], ItemStack::new(54, 1, 0))); // chest

    recipes.register_shaped(ShapedRecipe::new([[Some(cobble.clone()), Some(cobble.clone()), Some(cobble.clone())],[Some(cobble.clone()), None, Some(cobble.clone())],[Some(cobble.clone()), Some(cobble.clone()), Some(cobble.clone())]], ItemStack::new(61, 1, 0))); // furnace
}

fn pickaxe_shape(item: ItemStack) -> Grid {
    let sticks = ItemStack::new(280, 1, 0);
    let grid = [[Some(item.clone()), Some(item.clone()), Some(item.clone())], [None, Some(sticks.clone()), None], [None, Some(sticks.clone()), None]];
    grid
}

fn register_tools(recipes: &mut Solver) {
    register_axes(recipes);
    register_hoes(recipes);
    register_pickaxes(recipes);
}
fn register_pickaxes(recipes: &mut Solver) {
    let planks = ItemStack::new(5, 1, 0);
    recipes.register_shaped(ShapedRecipe::new(pickaxe_shape(planks.clone()), ItemStack::new(270, 1, 0)));

    let cobble = ItemStack::new(4, 1, 0);
    recipes.register_shaped(ShapedRecipe::new(pickaxe_shape(cobble.clone()), ItemStack::new(274, 1, 0)));
}
fn register_axes(recipes: &mut Solver) {
    let wood_axe_shapes = axe_shapes(ItemStack::new(5, 1, 0));
    let wood_axe = ItemStack::new(271, 1, 0);
    for shape in wood_axe_shapes {
        recipes.register_shaped(ShapedRecipe::new(shape, wood_axe.clone()));
    }

    let axe_shapes = axe_shapes(ItemStack::new(4, 1, 0));
    let axe = ItemStack::new(275, 1, 0);
    for shape in axe_shapes {
        recipes.register_shaped(ShapedRecipe::new(shape, axe.clone()));
    }
}

fn register_hoes(recipes: &mut Solver) {
    let wood_hoe_shapes = hoe_shapes(ItemStack::new(5, 1, 0));
    let wood_hoe = ItemStack::new(290, 1, 0);
    for shape in wood_hoe_shapes {
        recipes.register_shaped(ShapedRecipe::new(shape, wood_hoe.clone()));
    }

    let hoe_shapes = hoe_shapes(ItemStack::new(4, 1, 0));
    let hoe = ItemStack::new(291, 1, 0);
    for shape in hoe_shapes {
        recipes.register_shaped(ShapedRecipe::new(shape, hoe.clone()));
    }
}

fn hoe_shapes(item: ItemStack) -> Vec<Grid> {
    let mut grids = Vec::new();
    let sticks = ItemStack::new(280, 1, 0);
    let mut grid = [[Some(item.clone()), Some(item.clone()), None], [None, Some(sticks.clone()), None], [None, Some(sticks.clone()), None]];
    normalize(&mut grid);
    grids.push(grid);
    let mut grid = [[None, Some(item.clone()), Some(item.clone())], [None, Some(sticks.clone()), None], [None, Some(sticks.clone()), None]];
    normalize(&mut grid);
    grids.push(grid);
    grids
}

fn axe_shapes(item: ItemStack) -> Vec<Grid> {
    let mut grids = Vec::new();
    let sticks = ItemStack::new(280, 1, 0);
    let mut grid = [[Some(item.clone()), Some(item.clone()), None], [Some(item.clone()), Some(sticks.clone()), None], [None, Some(sticks.clone()), None]];
    normalize(&mut grid);
    grids.push(grid);
    let mut grid = [[None, Some(item.clone()), Some(item.clone())], [None, Some(sticks.clone()), Some(item.clone())], [None, Some(sticks.clone()), None]];
    normalize(&mut grid);
    grids.push(grid);
    grids
}