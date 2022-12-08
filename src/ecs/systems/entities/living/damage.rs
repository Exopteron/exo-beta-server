use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::living::{Health, EntityWorldInteraction}}, game::{Game, Position, DamageType}};

pub fn init_systems(g: &mut Game, s: &mut SystemExecutor<Game>) -> anyhow::Result<()> {
    s.add_system(void_damage);
    Ok(())
}


fn void_damage(g: &mut Game) -> SysResult {
    for (_, (hp, pos, dmg)) in g.ecs.query::<(&mut Health, &Position, &mut EntityWorldInteraction)>().iter() {
        if pos.y < -64.0 {
            if dmg.last_void_damage == 0 {
                hp.damage(2, DamageType::Void);
                dmg.last_void_damage = 10;
            } else {
                dmg.last_void_damage -= 1;
            }
        }
    }
    Ok(())
}