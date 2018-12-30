use crate::{
    util::Point2,
    io::tex::{Assets, Sprite},
    obj::{
        player::Player,
        enemy::Enemy,
        health::Health,
        bullet::Bullet,
        weapon::{WeaponInstance, WeaponDrop, WEAPONS},
        pickup::Pickup,
    }
};
use ggez::{
    Context, GameResult,
    graphics,
    error::GameError,
};

use std::path::Path;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};

use ::bincode;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug)]
/// All the objects in the current world
pub struct World {
    pub player: Player,
    pub grid: Grid,
    pub exit: Option<Point2>,
    pub intels: Vec<Point2>,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet<'static>>,
    pub weapons: Vec<WeaponDrop<'static>>,
    pub pickups: Vec<Pickup>,
}

impl World {
    pub fn enemy_pickup(&mut self) {
        for enemy in &mut self.enemies {
            let mut dead = None;
            for (w, weapon) in self.weapons.iter().enumerate() {
                if (weapon.pos - enemy.obj.pos).norm() <= 16. {
                    dead = Some(w);
                    break;
                }
            }
            if let Some(i) = dead {
                enemy.wep = Some(WeaponInstance::from_drop(self.weapons.remove(i)));
            }
            let mut deads = Vec::new();
            for (p, pickup) in self.pickups.iter().enumerate() {
                if (pickup.pos - enemy.obj.pos).norm() <= 16. {
                    deads.push(p);
                    break;
                }
            }
            for i in deads.into_iter() {
                let pickup = self.pickups.remove(i);
                pickup.apply(&mut enemy.health);
            }
        }
    }
    pub fn player_pickup(&mut self) {
        let player = &mut self.player;
        if player.wep.is_none() {
            let mut dead = None;
            for (w, weapon) in self.weapons.iter().enumerate() {
                if (weapon.pos - player.obj.pos).norm() <= 16. {
                    dead = Some(w);
                    break;
                }
            }
            if let Some(i) = dead {
                player.wep = Some(WeaponInstance::from_drop(self.weapons.remove(i)));
            }
        }

        let mut deads = Vec::new();
        for (p, pickup) in self.pickups.iter().enumerate() {
            if (pickup.pos - player.obj.pos).norm() <= 16. {
                deads.push(p);
                break;
            }
        }
        for i in deads.into_iter() {
            let pickup = self.pickups.remove(i);
            pickup.apply(&mut player.health);
        }
    }
}

pub struct Statistics {
    pub hits: usize,
    pub misses: usize,
    pub enemies_left: usize,
    pub health_left: Health,
    pub level: Level,
    pub weapon: Option<WeaponInstance<'static>>,
}

include!("material_macro.rs");

mat!{
    MISSING = Missing
    Grass = 0, Grass, false,
    Wall = 1, Wall, true,
    Floor = 2, Floor, false,
    Dirt = 3, Dirt, false,
    Asphalt = 4, Asphalt, false,
    Sand = 5, Sand, false,
    Concrete = 6, Concrete, true,
    WoodFloor = 7, WoodFloor, false,
    Missing = 255, Missing, true,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub grid: Grid,
    pub start_point: Option<Point2>,
    pub enemies: Vec<Enemy>,
    pub exit: Option<Point2>,
    pub intels: Vec<Point2>,
    pub pickups: Vec<(Point2, u8)>,
    pub weapons: Vec<WeaponDrop<'static>>,
}

impl Level {
    pub fn new(width: u16, height: u16) -> Self {
        Level {
            grid: Grid::new(width, height),
            start_point: None,
            enemies: Vec::new(),
            exit: None,
            intels: Vec::new(),
            pickups: Vec::new(),
            weapons: Vec::new(),
        }
    }
    pub fn load<P: AsRef<Path>>(path: P) -> GameResult<Self> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut ret = Level::new(0, 0);

        loop {
            let mut buf = String::with_capacity(16);
            reader.read_line(&mut buf)?;
            match &*buf.trim_right() {
                "" => continue,
                "GRD" => ret.grid = bincode::deserialize_from(&mut reader)
                .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "GRID" => {
                    let (w, grid): (usize, Vec<u16>) = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
                    ret.grid = Grid {
                        mats: grid.into_iter().map(|n| Material::from(n as u8)).collect(),
                        width: w as u16
                    }
                }
                "START" => ret.start_point = Some(
                    bincode::deserialize_from(&mut reader)
                        .map(|(x, y)| Point2::new(x, y))
                        .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?
                ),
                "ENEMIES" => ret.enemies = bincode::deserialize_from(&mut reader)
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "POINT GOAL" => ret.exit = Some(bincode::deserialize_from(&mut reader)
                    .map(|(x, y)| Point2::new(x, y))
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?),
                "INTELS" => ret.intels = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<(f32, f32)>| l.into_iter().map(|(x, y)| Point2::new(x, y)).collect())
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "PICKUPS" => ret.pickups = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<((f32, f32), u8)>| l.into_iter().map(|((x, y), i)| (Point2::new(x, y), i)).collect())
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "WEAPONS" => ret.weapons = bincode::deserialize_from(&mut reader)
                    .map(|l: Vec<((f32, f32), u8)>| l.into_iter().map(|((x, y), i)| WEAPONS[i as usize].make_instance().into_drop(Point2::new(x, y))).collect())
                    .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?,
                "END" => break,
                _ => return Err("Bad section".to_string())?
            }
        }

        Ok(ret)
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> GameResult<()> {
        let mut file = File::create(path)?;

        writeln!(file, "GRD")?;
        bincode::serialize_into(&mut file, &self.grid)
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        if let Some(start) = self.start_point {
            writeln!(file, "\nSTART")?;
            bincode::serialize_into(&mut file, &(start.x, start.y))
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if !self.enemies.is_empty() {
            writeln!(file, "\nENEMIES")?;
            bincode::serialize_into(&mut file, &self.enemies)
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if let Some(p) = self.exit {
            writeln!(file, "\nPOINT GOAL")?;
            bincode::serialize_into(&mut file, &(p.x, p.y))
            .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if !self.intels.is_empty() {
            writeln!(file, "\nINTELS")?;
            let intels: Vec<_> = self.intels.iter().map(|p| (p.x, p.y)).collect();
            bincode::serialize_into(&mut file, &intels)
                .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if !self.pickups.is_empty() {
            writeln!(file, "\nPICKUPS")?;
            let pickups: Vec<_> = self.pickups.iter().map(|&(p, i)| ((p.x, p.y), i)).collect();
            bincode::serialize_into(&mut file, &pickups)
                .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }
        if !self.weapons.is_empty() {
            writeln!(file, "\nWEAPONS")?;
            let pickups: Vec<((f32, f32), u8)> = self.weapons.iter().map(|w| ((w.pos.x, w.pos.y), {
                let mut index = 0;
                for (i, wep) in WEAPONS.iter().enumerate() {
                    if wep.name == w.weapon.name {
                        index = i;
                        break
                    }  
                }
                index as u8
            })).collect();
            bincode::serialize_into(&mut file, &pickups)
                .map_err(|e| GameError::UnknownError(format!("{:?}", e)))?;
        }

        writeln!(file, "\nEND")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid{
    width: u16,
    mats: Vec<Material>,
}

impl Grid {
    pub fn new(width: u16, height: u16) -> Self {
        Grid {
            width,
            mats: vec![Material::Grass; (width*height) as usize],
        }
    }
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }
    pub fn height(&self) -> u16 {
        self.mats.len() as u16 / self.width
    }
    pub fn widen(&mut self) {
        let width = self.width as usize;
        let height = self.height() as usize;
        self.mats.reserve_exact(height);
        for i in (1..=height).rev().map(|i| i * width) {
            self.mats.insert(i, Material::Grass);
        }
        self.width += 1;
    }
    pub fn thin(&mut self) {
        if self.width <= 1 {
            return
        }
        let width = self.width;
        for i in (1..=self.height()).rev().map(|i| i * width - 1) {
            self.mats.remove(i as usize);
        }
        self.width -= 1;
    }
    pub fn heighten(&mut self) {
        let new_len = self.mats.len() + self.width as usize;
        self.mats.reserve_exact(self.width as usize);
        self.mats.resize(new_len, Material::Grass);
    }
    pub fn shorten(&mut self) {
        let new_len = self.mats.len() - self.width as usize;
        if new_len == 0 {
            return
        }
        self.mats.truncate(new_len);
    }
    #[inline]
    pub fn snap(c: Point2) -> (u16, u16) {
        Self::snap_coords(c.x, c.y)
    }
    #[inline]
    fn idx(&self, x: u16, y: u16) -> usize {
        x.saturating_add(y.saturating_mul(self.width)) as usize
    }
    pub fn snap_coords(x: f32, y: f32) -> (u16, u16) {
        let x = (x / 32.) as u16;
        let y = (y / 32.) as u16;

        (x, y)
    }
    pub fn get(&self, x: u16, y: u16) -> Option<Material> {
        if x < self.width {
            self.mats.get(self.idx(x, y)).cloned()
        } else {
            None
        }
    }
    pub fn is_solid(&self, x: u16, y: u16) -> bool {
        self.get(x, y).map(|m| m.solid()).unwrap_or(true)
    }
    pub fn insert(&mut self, x: u16, y: u16, mat: Material) {
        if x < self.width {
            let i = self.idx(x, y);
            if let Some(m) = self.mats.get_mut(i) {
                *m = mat;
            }
        }
    }
    pub fn draw(&self, ctx: &mut Context, assets: &Assets) -> GameResult<()> {
        for (i, mat) in self.mats.iter().enumerate() {
            let x = f32::from(i as u16 % self.width) * 32.;
            let y = f32::from(i as u16 / self.width) * 32.;

            mat.draw(ctx, assets, x, y)?;
        }
        Ok(())
    }
}
