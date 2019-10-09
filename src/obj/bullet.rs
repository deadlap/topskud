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
    pub target: Point2,
    in_enemy: Option<usize>,
    in_wall: Option<u8>,
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
            in_wall: None,
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
        let d_vel = BULLET_DECCELERATION * DELTA * angle_to_vec(self.obj.rot);
        let d_pos = DELTA * self.vel - 0.5 * DELTA * d_vel;
        self.vel -= d_vel;
        if let BulletType::SawBlade = self.weapon.bullet_type {
            self.obj.rot += 0.08;
        }
        if self.vel.norm() < 375. {
            return Hit::Wall;
        }
        // Tjek om kuglen rammer spilleren
        // Da distancen kuglen rejser er et linjestykke, og spilleren er en cirkel, bruger vi `dist_line_circle`
        if Grid::dist_line_circle(start, d_pos, player.obj.pos) <= 16. {
            let hs = self.apply_damage(&mut player.health, player.obj.pos);
            return Hit::Player(hs);
        }
        // Tjek om kuglen rammer en enemy
        // Samme fremgangsmåde som med spilleren
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
        if !self.weapon.bullet_type.bouncy() {
            let (bx,by) = Grid::snap(start+d_pos);
            let r_mat = Grid::get(grid, bx, by);
            let (nx,ny) = Grid::snap(start);
            let n_mat = Grid::get(grid, nx, ny);
            if !cast.full(){
                if Grid::is_solid(grid, palette, bx, by) {
                    self.in_wall = r_mat;
                } else if Grid::is_solid(grid, palette, nx, ny) {
                    self.in_wall = n_mat;
                } else {
                    self.in_wall = None;
                }
            }
            if self.in_wall.is_some() {
                let rob_mat = palette.get_robust(self.in_wall.unwrap());
                let mat_req = (self.vel.norm()*(1.0-rob_mat) / self.weapon.bullet_speed)*self.weapon.penetration;
                if rob_mat <= mat_req {
                    self.vel *= 1.0-(1.0-self.weapon.penetration)*rob_mat;//1.0-rob_mat;
                    self.obj.pos += d_pos;
                    return Hit::None
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
    Player(bool),
    Enemy(usize, bool),
    None,
}