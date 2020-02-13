use ggez::{Context, GameResult, graphics::WHITE};

use crate::{
    util::{Vector2, angle_from_vec},
    game::{
        DELTA,
        world::{Grid, Palette},
    },
    io::tex::{Assets, }
};
use super::{Object, player::Player, enemy::Enemy, health::Health, weapon::{Weapon, BulletType}};

#[derive(Debug, Clone)]
pub struct Bullet<'a> {
    pub obj: Object,
    pub vel: Vector2,
    pub weapon: &'a Weapon,
    pub in_enemy: Option<usize>,
}

impl Bullet<'_> {
    pub fn apply_damage(&self, health: &mut Health) {
        let dmg = self.weapon.damage * self.vel.norm() / self.weapon.bullet_speed;

        health.weapon_damage(dmg, self.weapon.penetration);
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets, palette: &Palette, grid: &Grid) -> GameResult<()> {
        if let BulletType::Laser = self.weapon.bullet_type {
            let mut bullet_obj = self.obj.clone();
            let bullet_length = self.vel.normalize() * 8.;
            let img = a.get_img(ctx, self.weapon.bullet_type.get_spr());

            // Så længe vi ikke har ramt noget.
            loop {
                bullet_obj.draw(ctx, &*img, WHITE)?;

                let cast = grid.ray_cast(palette, bullet_obj.pos, bullet_length, true);
                if !cast.full() {
                    break;
                }
                bullet_obj.pos = cast.into_point();
            }
            
            Ok(())
        } else {
            let img = a.get_img(ctx, self.weapon.bullet_type.get_spr());
            self.obj.draw(ctx, &*img, WHITE)
        }
    }
    pub fn update(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Hit {
        let start = self.obj.pos;
        let d_pos = self.vel * DELTA;

        let mut velocity_decrease: f32 = 220. * DELTA;

        if let BulletType::SawBlade = self.weapon.bullet_type {
            velocity_decrease = 60. * DELTA;
            self.obj.rot += 0.08;
        }
        if self.vel.norm() < velocity_decrease {
            return Hit::Wall;
        }

        // Decrease velocity after damage could've been dealt
        self.vel -= self.vel.normalize() * velocity_decrease;

        // Ray cast bullet to see if we've hit a wall and move bullet accordingly
        let cast = grid.ray_cast(palette, start, d_pos, true);
        // Tjek om kuglen rammer spilleren
        // Da distancen kuglen rejser er et linjestykke, og spilleren er en cirkel, bruger vi `dist_line_circle`
        if Grid::dist_line_circle(start, d_pos, player.obj.pos) <= 16. {
            self.apply_damage(&mut player.health);
            return Hit::Player;
        }
        // Tjek om kuglen rammer en enemy
        // Samme fremgangsmåde som med spilleren
        for (i, enem) in enemies.iter_mut().enumerate() {
            if Grid::dist_line_circle(start, d_pos, enem.pl.obj.pos) <= 16. {
                if self.in_enemy.is_none() {
                    self.apply_damage(&mut enem.pl.health);
                    self.in_enemy = Some(i);
                    return Hit::Enemy(i);
                }
            } else if let Some(j) = self.in_enemy {
                if i == j {
                    self.in_enemy = None;
                }
            }
        }
        self.obj.pos = cast.into_point();
        if self.weapon.bullet_type.bouncy() {
            // to_wall er en vektor fra bulletens nuværende position til dér, hvor den vil ramme væggen.
            if let Some(to_wall) = cast.half_vec() {
                self.in_enemy = None;

                let clip = cast.clip();
                self.obj.pos += clip -  2. * clip.dot(&to_wall)/to_wall.norm_squared() * to_wall;
                self.vel -= 2. * self.vel.dot(&to_wall)/to_wall.norm_squared() * to_wall;
                self.obj.rot = angle_from_vec(self.vel);
            }
            // Hvis bulleten er bouncy, rammer den aldrig en væg.
            Hit::None
        } else if cast.full() {
            // Hvis castet er fuldt, har den ikke ramt noget
            Hit::None
        } else {
            Hit::Wall
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Hit {
    Wall,
    Player,
    Enemy(usize),
    None,
}