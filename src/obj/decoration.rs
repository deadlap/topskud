use crate::{
    io::tex::{Assets, },
};
use ggez::{Context, GameResult, graphics::Color};

use super::Object;

#[derive(Debug, Clone, Deserialize)]
pub struct OldDecoration {
    pub obj: Object,
    pub i: usize,
}

impl OldDecoration {
    pub fn renew(self) -> Decoration {
        let OldDecoration{obj, i} = self;

        Decoration {
            obj,
            spr: OLD_DECORATION_LIST[i].into()
        }
    }
}

const OLD_DECORATION_LIST: [&str; 15] = [
    "decorations/chair1",
    "decorations/chair2",
    "decorations/chair_boss",
    "decorations/lamp_post",
    "decorations/office_plant",
    "decorations/office_plant2",
    "decorations/office_plant3",
    "decorations/trashcan",
    "decorations/manhole_cover",
    "decorations/manhole_cover2",
    "decorations/desk_lamp",
    "decorations/wall_light",
    "decorations/wall_light2",
    "decorations/wall_light3",
    "decorations/road_mark"
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decoration {
    pub obj: Object,
    pub spr: Box<str>,
}

impl Decoration {
    #[inline]
    pub fn new<S: Into<Box<str>>>(obj: Object, spr: S) -> Self {
        Decoration {
            obj,
            spr: spr.into()
        }
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets, color: Color) -> GameResult<()> {
        let img = a.get_img(ctx, &self.spr);
        self.obj.draw(ctx, &*img, color)
    }
    #[inline]
    pub fn is_solid(&self) -> bool {
        &*self.spr == "decorations/trashcan"
    }
}