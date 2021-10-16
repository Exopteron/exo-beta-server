// Basically from feather-rs but made worse. Full credit to them and caelunshun <caelunshun@gmail.com>. (sorry)
use super::*;
pub const TABLE_WIDTH: usize = 3;
pub const TABLE_SIZE: usize = TABLE_WIDTH * TABLE_WIDTH;
pub type Grid = [[Option<ItemStack>; TABLE_WIDTH]; TABLE_WIDTH];

#[derive(Clone, Debug)]
pub struct ShapedRecipe {
    pub input: Grid,
    /// Output item stack.
    pub output: ItemStack,
}

#[derive(Clone, Debug)]
pub struct ShapelessRecipe {
    /// The set of input items required.
    /// Must be a sorted vector to allow for efficient
    /// comparison.
    pub input: ArrayVec<[ItemStack; TABLE_SIZE]>,
    /// Output item stack.
    pub output: ItemStack,
}
impl ShapelessRecipe {
    pub fn new(mut input: ArrayVec<[ItemStack; TABLE_SIZE]>, output: ItemStack) -> Self {
        Self { input, output }
    }
}
use arrayvec::*;
#[derive(Debug, Clone, Default)]
pub struct Solver {
    shaped: HashMap<Grid, ItemStack>,
    shapeless: HashMap<ArrayVec<[ItemStack; TABLE_SIZE]>, ItemStack>,
}

#[derive(Clone, Debug)]
pub enum Recipe {
    Shaped(ShapedRecipe),
    Shapeless(ShapelessRecipe),
}

impl Solver {
    pub fn new() -> Self {
        log::info!("[Feather Crafting Solver] Initializing crafting solver");
        Self::default()
    }

    /// Given an input crafting grid, attempts to find
    /// an item to craft. Returns the craftable item stack,
    /// or `None` if the grid satisfies no recipes.
    pub fn solve(&self, input: &mut Grid) -> Option<ItemStack> {
        normalize(input);
        for row in input.iter_mut() {
            for item in row.iter_mut() {
                if let Some(item) = item {
                    item.count = 1;
                }
            }
        }
        // Try shaped first, then shapeless.
        if let Some(output) = self.shaped.get(input).copied() {
            Some(output)
        } else {
            // Sort inputs to perform a shapeless lookup.
            let mut inputs = ArrayVec::new();
            input
                .iter()
                .flatten()
                .filter_map(|slot| *slot)
                .for_each(|item| inputs.push(item));
            inputs.sort_unstable();
            log::info!("Inputs: {:?}", inputs);
            if let Some(output) = self.shapeless.get(&inputs).copied() {
                log::info!("There is some!");
                Some(output)
            } else {
                log::info!("None, loser!");
                None
            }
        }
    }

    /// Registers a recipe with this `Solver`. Future calls to `solve()`
    /// will account for the new recipe.
    pub fn register(&mut self, recipe: Recipe) {
        match recipe {
            Recipe::Shaped(shaped) => self.register_shaped(shaped),
            Recipe::Shapeless(shapeless) => self.register_shapeless(shapeless),
        }
    }

    fn register_shaped(&mut self, shaped: ShapedRecipe) {
        //log::info!("[Feather Crafting Solver] Registering shaped crafting recipe for item {}", shaped.output.id);
        self.shaped.insert(shaped.input, shaped.output);
    }

    fn register_shapeless(&mut self, shapeless: ShapelessRecipe) {
        //log::info!("[Feather Crafting Solver] Registering shapeless crafting recipe for item {}", shapeless.output.id);
        self.shapeless.insert(shapeless.input, shapeless.output);
    }
}

fn is_empty(
    grid: &Grid,
    coord1: (usize, usize),
    coord2: (usize, usize),
    coord3: (usize, usize),
) -> bool {
    grid[coord1.0][coord1.1].is_none()
        && grid[coord2.0][coord2.1].is_none()
        && grid[coord3.0][coord3.1].is_none()
}

pub fn normalize(grid: &mut Grid) {
    // Find number of empty upper rows
    let mut y = 0;
    while is_empty(grid, (0, y), (1, y), (2, y)) && y < TABLE_WIDTH - 1 {
        y += 1;
    }

    // Find number of empty leftmost columns
    let mut x = 0;
    while is_empty(grid, (x, 0), (x, 1), (x, 2)) && x < TABLE_WIDTH - 1 {
        x += 1;
    }

    translate(grid, x, y);
}
pub fn translate(grid: &mut Grid, x: usize, y: usize) {
    // Translate to the left
    grid.copy_within(x..TABLE_WIDTH, 0);
    // Upward
    for column in grid.iter_mut() {
        column.copy_within(y..TABLE_WIDTH, 0);
    }

    // Fill in newly unused space with empty slots
    for item in grid.iter_mut().skip(TABLE_WIDTH - x) {
        *item = [None, None, None];
    }
    for column in TABLE_WIDTH - y..TABLE_WIDTH {
        for row in grid.iter_mut() {
            row[column] = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shaped() {
        let mut solver = Solver::new();
        let mut grid = Grid::default();
        grid[0][0] = Some(ItemStack::new(1, 0, 1));
        solver.register(Recipe::Shaped(ShapedRecipe { input: grid, output: ItemStack::new(5, 0, 1) }));
        let mut grid = Grid::default();
        grid[0][0] = Some(ItemStack::new(1, 0, 1));
        panic!("{:?}", solver.solve(&mut grid));
    }
    #[test]
    fn shapeless() {
        let mut arrvec = ArrayVec::new();
        arrvec.push(ItemStack::new(4, 0, 1));
        let shapeless = ShapelessRecipe { input: arrvec, output: ItemStack::new(3, 0, 1) };
        let mut solver = Solver::new();
        solver.register(Recipe::Shapeless(shapeless));
        let mut grid = Grid::default();
        grid[0][1] = Some(ItemStack::new(4, 0, 1));
        panic!("{:?}", solver.solve(&mut grid));
    }
}