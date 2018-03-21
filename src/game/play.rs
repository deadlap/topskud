use ::*;
use super::world::*;
use ggez::graphics::{Drawable, DrawMode, Color, WHITE, Rect};
use ggez::graphics::spritebatch::SpriteBatch;

/// The state of the game
pub struct Play {
    health: u8,
    world: World,
    holes: SpriteBatch,
}

impl Play {
    pub fn new(level: Level, a: &Assets) -> Self {
        Play {
            health: 10,
            world: World {
                enemies: level.enemies,
                bullets: Vec::new(),
                player: Object::new(level.start_point.unwrap_or(Point2::new(500., 500.))),
                grid: level.grid,
            },
            holes: SpriteBatch::new(a.get_img(Sprite::Hole).clone()),
        }
    }
}

impl GameState for Play {
    fn update(&mut self, s: &mut State) {
        let mut deads = Vec::new();
        for (i, bullet) in self.world.bullets.iter_mut().enumerate().rev() {
            bullet.pos += 500. * DELTA * angle_to_vec(bullet.rot);
            if bullet.is_on_solid(&self.world.grid) {
                self.holes.add(bullet.drawparams());
                deads.push(i);
            } else if (bullet.pos-self.world.player.pos).norm() <= 16. {
                deads.push(i);
                self.health -= 1;
            }
        }
        for i in deads {
            self.world.bullets.remove(i);
        }

        let mut deads = Vec::new();
        for (e, enemy) in self.world.enemies.iter_mut().enumerate().rev() {
            if enemy.can_see(self.world.player.pos, &self.world.grid) {
                enemy.last_known_player_position = Some(self.world.player.pos);
            }
            enemy.update();
            let mut dead = None;
            for (i, bullet) in self.world.bullets.iter().enumerate().rev() {
                if (bullet.pos - enemy.obj.pos).norm() <= 16. {
                    dead = Some(i);
                    enemy.health -= 1;
                    if enemy.health == 0 {
                        deads.push(e);
                    }
                    break
                }
            }
            if let Some(i) = dead {
                self.world.bullets.remove(i);
            }
        }
        for i in deads {
            self.world.enemies.remove(i);
        }

        let speed = if s.modifiers.shift {
            300.
        } else {
            175.
        };
        self.world.player.move_on_grid(Vector2::new(s.input.hor(), s.input.ver()), speed, &self.world.grid);
    }
    fn logic(&mut self, s: &mut State, _ctx: &mut Context) {
        let dist = s.mouse - s.offset - self.world.player.pos;

        self.world.player.rot = angle_from_vec(&dist);

        // Center the camera on the player
        let p = self.world.player.pos;
        s.focus_on(p);
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, WHITE)?;
        self.world.grid.draw(ctx, &s.assets)?;
        graphics::set_color(ctx, Color{r:0.,g:0.,b:0.,a:1.})?;
        self.world.player.draw(ctx, s.assets.get_img(Sprite::Person))?;
        for enemy in &self.world.enemies {
            graphics::set_color(ctx, BLUE)?;
            enemy.draw_visibility_cone(ctx, 400.)?;

            if enemy.can_see(self.world.player.pos, &self.world.grid) {
                graphics::set_color(ctx, GREEN)?;
                graphics::line(ctx, &[enemy.obj.pos, self.world.player.pos], 3.)?;
            }
            enemy.draw(ctx, &s.assets)?;
        }
        graphics::set_color(ctx, WHITE)?;
        for bullet in &self.world.bullets {
            bullet.draw(ctx, s.assets.get_img(Sprite::Bullet))?;
        }
        self.holes.draw_ex(ctx, Default::default())?;

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        graphics::set_color(ctx, graphics::BLACK)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 1., y: 1., w: 102., h: 26.})?;
        graphics::set_color(ctx, GREEN)?;
        graphics::rectangle(ctx, DrawMode::Fill, Rect{x: 2., y: 2., w: self.health as f32 * 10., h: 24.})?;

        graphics::set_color(ctx, RED)?;
        let drawparams = graphics::DrawParam {
            dest: s.mouse,
            offset: Point2::new(0.5, 0.5),
            .. Default::default()
        };
        graphics::draw_ex(ctx, s.assets.get_img(Sprite::Crosshair), drawparams)
    }
    fn mouse_up(&mut self, _s: &mut State, _ctx: &mut Context, btn: MouseButton) {
        if let MouseButton::Left = btn {
            let pos = self.world.player.pos + 16. * angle_to_vec(self.world.player.rot);
            let mut bul = Object::new(pos);
            bul.rot = self.world.player.rot;

            self.world.bullets.push(bul);
        }
    }
}
