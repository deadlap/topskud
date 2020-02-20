use std::{option::IntoIter, iter::{Chain, IntoIterator}};

use ggez::{Context, GameResult, graphics::{self, WHITE, Color, Mesh, BlendMode, set_blend_mode, MeshBuilder, DrawMode, DrawParam, FillOptions}};
use std::f32::consts::PI;

use crate::{
    util::{Point2, angle_to_vec},
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
    game::{world::{Grid, Palette}},
};

use super::{Object, health::Health, weapon::{Weapon, WeaponInstance, WeaponSlot}, grenade::Utilities};

pub const VISIBILITY: f32 = ::std::f32::consts::FRAC_PI_4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub obj: Object,
    #[serde(skip)]
    pub wep: WepSlots,
    #[serde(skip)]
    pub health: Health,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveSlot {
    Knife = 0,
    Holster = 1,
    Holster2 = 2,
    Sling = 3,
}

impl ActiveSlot {
    #[inline]
    fn subtract(&mut self) {
        use self::ActiveSlot::*;
        *self = match *self {
            Knife => Knife,
            Holster => Knife,
            Holster2 => Holster,
            Sling => Holster2,
        };
    }
}

impl Default for ActiveSlot {
    #[inline(always)]
    fn default() -> Self {
        ActiveSlot::Knife
    }
}

#[derive(Debug, Default, Clone)]
pub struct WepSlots {
    pub active: ActiveSlot,
    pub utilities: Utilities,
    pub holster: Option<WeaponInstance<'static>>,
    pub holster2: Option<WeaponInstance<'static>>,
    pub sling: Option<WeaponInstance<'static>>,
}

impl WepSlots {
    #[inline(always)]
    pub fn slot_has_weapon(&self, new_active: ActiveSlot) -> bool {
        match new_active {
            ActiveSlot::Knife => true,
            ActiveSlot::Holster => self.holster.is_some(),
            ActiveSlot::Holster2 => self.holster2.is_some(),
            ActiveSlot::Sling => self.sling.is_some(),
        }
    }
    /// Set active to first weapon, falling back to knife
    pub fn init_active(&mut self) {
        self.active = match self {
            WepSlots{holster: Some(_), ..} => ActiveSlot::Holster,
            WepSlots{holster: None, holster2: Some(_), ..} => ActiveSlot::Holster2,
            WepSlots{holster: None, holster2: None, sling: Some(_), ..} => ActiveSlot::Sling,
            WepSlots{holster: None, holster2: None, sling: None, ..} => ActiveSlot::Knife,
        };
    }
    #[inline(always)]
    pub fn switch(&mut self, new_active: ActiveSlot) {
        if self.slot_has_weapon(new_active) {
            self.active = new_active;
        }
    }
    #[must_use]
    pub fn take_active(&mut self) -> Option<WeaponInstance<'static>> {
        let wep = match self.active {
            ActiveSlot::Knife => None,
            ActiveSlot::Holster => std::mem::take(&mut self.holster),
            ActiveSlot::Holster2 => std::mem::take(&mut self.holster2),
            ActiveSlot::Sling => std::mem::take(&mut self.sling),
        };
        while !self.slot_has_weapon(self.active) {
            self.active.subtract();
        }
        wep
    }
    #[inline(always)]
    pub fn get_active(&self) -> Option<&WeaponInstance<'static>> {
        match self.active {
            ActiveSlot::Knife => None,
            ActiveSlot::Holster => self.holster.as_ref(),
            ActiveSlot::Holster2 => self.holster2.as_ref(),
            ActiveSlot::Sling => self.sling.as_ref(),
        }
    }
    #[inline(always)]
    pub fn get_active_mut(&mut self) -> Option<&mut WeaponInstance<'static>> {
        match self.active {
            ActiveSlot::Knife => None,
            ActiveSlot::Holster => self.holster.as_mut(),
            ActiveSlot::Holster2 => self.holster2.as_mut(),
            ActiveSlot::Sling => self.sling.as_mut(),
        }
    }
    #[must_use]
    pub fn insert(&mut self, weapon: &Weapon) -> &mut Option<WeaponInstance<'static>> {
        match (weapon.slot, self) {
            (WeaponSlot::Holster, WepSlots{holster: ref mut s @ None, ..}) |
            (WeaponSlot::Holster, WepSlots{holster2: ref mut s @ None, ..}) |
            (WeaponSlot::Holster, WepSlots{active: ActiveSlot::Holster, holster: ref mut s, ..}) |
            (WeaponSlot::Holster, WepSlots{active: ActiveSlot::Holster2, holster2: ref mut s, ..}) |
            (WeaponSlot::Holster, WepSlots{holster2: ref mut s, ..}) |
            (WeaponSlot::Sling, WepSlots{sling: ref mut s, ..}) => s,
        }
    }
    #[must_use]
    #[inline]
    pub fn add_weapon(&mut self, wep_instance: WeaponInstance<'static>) -> Option<WeaponInstance<'static>> {
        std::mem::replace(self.insert(&wep_instance.weapon), Some(wep_instance))
    }
}

impl IntoIterator for WepSlots {
    type IntoIter = Chain<
        Chain<IntoIter<WeaponInstance<'static>>, IntoIter<WeaponInstance<'static>>>,
        IntoIter<WeaponInstance<'static>>,
    >;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        #[allow(clippy::unneeded_field_pattern)]
        let WepSlots{active: _, utilities: _, holster, holster2, sling} = self;

        holster.into_iter().chain(holster2).chain(sling)
    }
}

impl Player {
    #[inline]
    pub fn new(obj: Object) -> Self {
        Self {
            obj,
            wep: Default::default(),
            health: Health::default(),
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
    pub fn with_weapon(self, wep: WepSlots) -> Self {
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
        {
            let hands_sprite = if let Some(wep) = self.wep.get_active() {
                wep.weapon.hands_sprite
            } else {
                "weapons/knife_hands"
            };

            let dp = graphics::DrawParam {
                dest: (self.obj.pos+angle_to_vec(self.obj.rot)*16.).into(),
                color,
                .. self.obj.drawparams()
            };
            let img = a.get_img(ctx, hands_sprite);
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
        
        let mut p_added = false; //is the player position added yet.
        let mut fpoint = Point2::new(0.,0.); // the first point (will be changed)
        // add all the corners of level to the background/fog mesh
        // let mut screen = Vec::new();
        let mut screen = Vec::new();
        screen.push(Point2::new(0., lvl_height));
        screen.push(Point2::new(0., 0.));
        screen.push(Point2::new(lvl_width, 0.));
        screen.push(Point2::new(lvl_width, lvl_height));
        for i in 0..(angle as u16)/2{
            let cast = grid.ray_cast(palette, pos, angle_to_vec((start_angle + (i*2) as f32)*PI/180.)*length, true);
            let current_point = cast.into_point()+angle_to_vec((start_angle + (i*2) as f32)*PI/180.)*15.;
            if i == 0 {fpoint = current_point;}
            if (current_point.y < pos.y || current_point.x < pos.x) && !p_added && i == 0 {
                p_added = true;
                screen.push(pos);
                fpoint = pos;
            }
            screen.push(current_point);
        }
        screen.push(pos);
        screen.push(fpoint);
        screen.push(Point2::new(lvl_width, lvl_height));
        screen.push(Point2::new(0., lvl_height));
        let mesh_screen = Mesh::new_polygon(ctx, DrawMode::Fill(FillOptions::even_odd()), &screen, Color::from_rgba(4, 6, 6, 255))?;
        graphics::draw(ctx, &mesh_screen, DrawParam::default())
    }

    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if let Some(wep) = self.wep.get_active_mut() {
            wep.update(ctx, mplayer)?;
        }
        Ok(())
    }
}
