use ggez::{Context, GameResult, graphics::{self, WHITE, Color, Mesh, DrawMode, DrawParam}};
use std::f32::consts::PI;

use crate::{
    util::{Point2, angle_to_vec},
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
    game::{world::{Grid, Palette}},
};

use super::{Object, health::Health, weapon::WeaponInstance, grenade::Utilities};

pub const VISIBILITY: f32 = ::std::f32::consts::FRAC_PI_4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub obj: Object,
    #[serde(skip)]
    pub wep: Option<WeaponInstance<'static>>,
    #[serde(skip)]
    pub health: Health,
    #[serde(skip)]
    pub utilities: Utilities,
}

impl Player {
    #[inline]
    pub fn new(obj: Object) -> Self {
        Self {
            obj,
            wep: None,
            health: Health::default(),
            utilities: Utilities::default(),
        }
    }
    #[inline]
    pub fn from_point(p: Point2) -> Self {
        Player::new(Object::new(p))
    }
    #[inline]
    pub fn with_health(self, health: Health) -> Self {
        Self {
            health,
            .. self
        }
    }
    #[inline]
    pub fn with_weapon(self, wep: Option<WeaponInstance<'static>>) -> Self {
        Self {
            wep,
            .. self
        }
    }

    #[inline]
    pub fn draw_player(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.draw(ctx, a, "common/player", WHITE)
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets, sprite: &str, color: Color) -> GameResult<()> {
        if let Some(wep) = self.wep {
            let dp = graphics::DrawParam {
                dest: (self.obj.pos+angle_to_vec(self.obj.rot)*16.).into(),
                color,
                .. self.obj.drawparams()
            };

            let img = a.get_img(ctx, &wep.weapon.hands_sprite);
            graphics::draw(ctx, &*img, dp)?;
        }
        let img = a.get_img(ctx, sprite);
        self.obj.draw(ctx, &*img, color)
    }
    pub fn draw_visible_area(&self, ctx: &mut Context, length: f32, palette: &Palette, grid: &Grid) -> GameResult<()> {
        let lvl_height = (grid.height() as f32)*32.;
        let lvl_width = (grid.width() as f32)*32.;
        let Object{pos, rot} = self.obj;
        let dir1 = angle_to_vec(rot - VISIBILITY - PI/12.);
        let dir2 = angle_to_vec(rot + VISIBILITY + PI/12.);
        let angle = ((dir1.angle(&dir2)*180.)/PI).floor();
        let start_angle = ((rot - VISIBILITY - PI/12.)*180.)/PI;
        let mut list_of_points = Vec::new();
        list_of_points.push(pos);
        
        let mut screen = Vec::new();
        let mut p_added = false;
        let mut fpoint = Point2::new(0.,0.);
        screen.push(Point2::new(0., lvl_height));
        screen.push(Point2::new(0., 0.));
        screen.push(Point2::new(lvl_width, 0.));
        screen.push(Point2::new(lvl_width, lvl_height));
        for i in 0..(angle as u16){
            let cast = grid.ray_cast(palette, pos, angle_to_vec((start_angle + i as f32)*PI/180.)*length, true);
            let current_point = cast.into_point()+angle_to_vec((start_angle + i as f32)*PI/180.)*20.;
            list_of_points.push(current_point);
            if i == 0 {fpoint = current_point;}
            if (current_point.y < pos.y || current_point.x < pos.x) && !p_added && i == 0 {
                p_added = true;
                screen.push(pos);
                fpoint = pos;
            }
            screen.push(current_point);
        }
        list_of_points.push(pos);
        screen.push(pos);        
        screen.push(fpoint);
        screen.push(Point2::new(lvl_width, lvl_height));
        screen.push(Point2::new(0., lvl_height));
        let mesh = Mesh::new_polyline(ctx, DrawMode::stroke(1.5), &list_of_points, Color::from_rgba(4, 6, 6, 245))?;
        let mesh2 = Mesh::new_ellipse(ctx, DrawMode::stroke(1.5), pos, 80., 80., 80., Color::from_rgba(4, 6, 6, 245))?;
        let mesh_screen = Mesh::new_polygon(ctx, DrawMode::fill(), &screen, Color::from_rgba(4, 6, 6, 245))?;
        graphics::draw(ctx, &mesh_screen, DrawParam::default())?;
        graphics::draw(ctx, &mesh2, DrawParam::default())?;
        graphics::draw(ctx, &mesh, DrawParam::default())
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if let Some(wep) = &mut self.wep {
            wep.update(ctx, mplayer)?;
        }
        Ok(())
    }
}
