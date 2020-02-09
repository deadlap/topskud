use ggez::{Context, GameResult, graphics::WHITE};

use crate::{
    util::{Point2, Vector2, angle_to_vec, angle_from_vec},
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
    in_enemy: Option<usize>,
    in_wall: Option<u8>,
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

        let mut velocity_decrease: f32 = 650. * DELTA;

        if let BulletType::SawBlade = self.weapon.bullet_type {
            velocity_decrease = 350. * DELTA;
            self.obj.rot += 0.08;
        }
        if self.vel.norm() < velocity_decrease {
            return Hit::Wall;
        }
        if self.vel.norm() <= velocity_decrease {
            return Hit::Wall;
        }
        // Check if we've hit a player or an enemy
        if Grid::dist_line_circle(start, d_pos, player.obj.pos) <= 16. {
            self.apply_damage(&mut player.health);
            return Hit::Player;
        }
        for (i, enem) in enemies.iter_mut().enumerate() {
            if Grid::dist_line_circle(start, d_pos, enem.pl.obj.pos) <= 16. {
                self.apply_damage(&mut enem.pl.health);
                return Hit::Enemy(i);
            }
        }

        // Decrease velocity after damage could've been dealt
        self.vel -= self.vel.normalize() * velocity_decrease;

        // Ray cast bullet to see if we've hit a wall and move bullet accordingly
        let cast = grid.ray_cast(palette, start, d_pos, true);
        if cast.full() || self.weapon.bullet_type.bouncy() {
            self.in_wall = None;
        } else {
            let dir = angle_to_vec(self.obj.rot);
            let (cur_x,cur_y) = Grid::snap(cast.into_point()+dir);
            let cur_mat = Grid::get(grid, cur_x, cur_y);
            if Grid::is_solid(grid, palette, cur_x, cur_y) {
                self.in_wall = cur_mat;
            } else {
                self.in_wall = None;
            }
        }
        if self.in_wall.is_none() {
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
        }
        self.obj.pos = cast.into_point();
        if self.weapon.bullet_type.bouncy() {
            // to_wall er en vektor fra bulletens nuværende position til dér, hvor den vil ramme væggen.
            if let Some(to_wall) = cast.half_vec() {
                self.in_enemy = None;

                let clip = cast.clip();
                self.obj.pos += clip -  2. * clip.dot(&to_wall)/to_wall.norm_squared() * to_wall;
                self.vel -= 2. * self.vel.dot(&to_wall)/to_wall.norm_squared() * to_wall - 2. * self.vel;
                self.obj.rot = angle_from_vec(self.vel);
            }
            // Hvis bulleten er bouncy, rammer den aldrig en væg.
            Hit::None
        } else if !self.weapon.bullet_type.bouncy() && self.in_wall.is_some() {
            let material_robustness = palette.get_robust(self.in_wall.unwrap());
            self.vel*=1.-material_robustness;
            if material_robustness <= self.weapon.penetration && self.vel.norm() >= velocity_decrease {
                self.obj.pos += cast.clip();
                Hit::None
            } else {
                Hit::Wall
            }
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