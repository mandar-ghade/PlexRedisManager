use strum_macros::{Display, EnumString};

#[derive(Clone, Copy, Debug, Display, EnumString)]
#[allow(non_camel_case_types)]
pub enum BoosterGroup {
    Arcade,
    Draw_My_Thing,
    Master_Builders,
    Speed_Builders,
    Block_Hunt,
    Cake_Wars,
    Survival_Games,
    Skywars,
    Bridges,
    MineStrike,
    Smash_Mobs,
    Champions,
    Nano_Games,
}
