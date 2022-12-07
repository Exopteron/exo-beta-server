use crate::{ecs::{systems::SystemExecutor, entities::living::{hostile::skeleton::SkeletonEntityBuilder}}, game::{Game, Position}, entity_loader::RegEntityNBTLoaders};

pub fn init_systems(g: &mut Game, s: &mut SystemExecutor<Game>) -> anyhow::Result<()> {
    
    let mut loaders = g.objects.get_mut::<RegEntityNBTLoaders>()?;
    loaders.insert("Skeleton", Box::new(|tag, builder| {

        let hp = tag.get_i16("Health").map_err(|_| anyhow::anyhow!("No tag"))?;

        SkeletonEntityBuilder::build(None, hp, builder);

        Ok(())
    }));

    Ok(())
}