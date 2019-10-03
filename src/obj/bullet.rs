use ggez::{Context, GameResult, graphics::WHITE};

use crate::{
    util::{Point2, Vector2, angle_to_vec, angle_from_vec},
    game::{
        DELTA,
        world::{Grid, Palette},
    },
    io::tex::{Assets, }
};
use super::{Object, player::Player, enemy::Enemy, health::Health, weapon::Weapon};

#[derive(Debug, Clone)]
pub struct Bullet<'a> {
    pub obj: Object,
    pub vel: Vector2,
    pub weapon: &'a Weapon,
    pub target: Point2,
    in_enemy: Option<usize>,
}

const HEADSHOT_BONUS: f32 = 1.5;

const BULLET_DECCELERATION: f32 = 25.7;

impl<'a> Bullet<'a> {
    pub fn new(obj: Object, weapon: &'a Weapon, target: Point2) -> Self {
        Bullet {
            vel: weapon.bullet_speed * angle_to_vec(obj.rot),
            obj,
            weapon,
            target,
            in_enemy: None,
        }
    }
    pub fn apply_damage(&self, health: &mut Health, pos: Point2) -> bool {
        let headshot = (self.target - pos).norm() <= 12.;
        let dmg = if headshot {
            HEADSHOT_BONUS
        } else {
            1.
        } * self.weapon.damage * (self.vel.norm() / self.weapon.bullet_speed);

        health.weapon_damage(dmg, self.weapon.penetration);
        headshot
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        let img = a.get_img(ctx, self.weapon.bullet_type.get_spr());
        self.obj.draw(ctx, &*img, WHITE)
    }
    pub fn update(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Hit {
        let start = self.obj.pos;
        let d_vel = BULLET_DECCELERATION * DELTA * angle_to_vec(self.obj.rot);

        let d_pos = DELTA * self.vel - 0.5 * DELTA * d_vel;
        self.vel -= d_vel;
        if self.vel.norm() < 375. {
            return Hit::Wall;
        }

        if Grid::dist_line_circle(start, d_pos, player.obj.pos) <= 16. {
            let hs = self.apply_damage(&mut player.health, player.obj.pos);
            return Hit::Player(hs);
        }
        for (i, enem) in enemies.iter_mut().enumerate() {
            if Grid::dist_line_circle(start, d_pos, enem.pl.obj.pos) <= 16. {
                if self.in_enemy.is_none() {
                    let hs = self.apply_damage(&mut enem.pl.health, enem.pl.obj.pos);
                    self.in_enemy = Some(i);
                    return Hit::Enemy(i, hs);
                }
            } else if let Some(j) = self.in_enemy {
                if i == j {
                    self.in_enemy = None;
                }
            }
        }
        let cast = grid.ray_cast(palette, start, d_pos, true);
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
    Player(bool),
    Enemy(usize, bool),
    None,
}