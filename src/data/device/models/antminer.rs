use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize, Display)]
pub enum AntMinerModel {
    #[serde(alias = "ANTMINER D3")]
    #[serde(alias = "d3")]
    D3,
    #[serde(alias = "ANTMINER HS3")]
    #[serde(alias = "hs3")]
    HS3,
    #[serde(alias = "ANTMINER L3+")]
    #[serde(alias = "l3plus")]
    #[serde(alias = "l3+")]
    L3Plus,
    #[serde(alias = "ANTMINER KA3")]
    #[serde(alias = "ka3")]
    KA3,
    #[serde(alias = "ANTMINER KS3")]
    #[serde(alias = "ks3")]
    KS3,
    #[serde(alias = "ANTMINER DR5")]
    #[serde(alias = "dr5")]
    DR5,
    #[serde(alias = "ANTMINER KS5")]
    #[serde(alias = "ks5")]
    KS5,
    #[serde(alias = "ANTMINER KS5 PRO")]
    #[serde(alias = "ks5pro")]
    KS5Pro,
    #[serde(alias = "ANTMINER L7")]
    #[serde(alias = "l7")]
    L7,
    #[serde(alias = "ANTMINER K7")]
    #[serde(alias = "k7")]
    K7,
    #[serde(alias = "ANTMINER D7")]
    #[serde(alias = "d7")]
    D7,
    #[serde(alias = "ANTMINER E9 PRO")]
    #[serde(alias = "e9pro")]
    E9Pro,
    #[serde(alias = "ANTMINER D9")]
    #[serde(alias = "d9")]
    D9,
    #[serde(alias = "ANTMINER S9")]
    #[serde(alias = "s9")]
    S9,
    #[serde(alias = "ANTMINER S9I")]
    #[serde(alias = "s9i")]
    S9i,
    #[serde(alias = "ANTMINER S9J")]
    #[serde(alias = "s9j")]
    S9j,
    #[serde(alias = "ANTMINER T9")]
    #[serde(alias = "t9")]
    T9,
    #[serde(alias = "ANTMINER L9")]
    #[serde(alias = "l9")]
    L9,
    #[serde(alias = "ANTMINER Z15")]
    #[serde(alias = "z15")]
    Z15,
    #[serde(alias = "ANTMINER Z15 PRO")]
    #[serde(alias = "z15pro")]
    Z15Pro,
    #[serde(alias = "ANTMINER S17")]
    #[serde(alias = "s17")]
    S17,
    #[serde(alias = "ANTMINER S17+")]
    #[serde(alias = "s17plus")]
    #[serde(alias = "s17+")]
    S17Plus,
    #[serde(alias = "ANTMINER S17 PRO")]
    #[serde(alias = "s17pro")]
    S17Pro,
    #[serde(alias = "ANTMINER S17E")]
    #[serde(alias = "s17e")]
    S17e,
    #[serde(alias = "ANTMINER T17")]
    #[serde(alias = "t17")]
    T17,
    #[serde(alias = "ANTMINER T17+")]
    #[serde(alias = "t17plus")]
    #[serde(alias = "t17+")]
    T17Plus,
    #[serde(alias = "ANTMINER T17E")]
    #[serde(alias = "t17e")]
    T17e,
    #[serde(alias = "ANTMINER S19")]
    #[serde(alias = "s19")]
    S19,
    #[serde(alias = "ANTMINER S19L")]
    #[serde(alias = "s19l")]
    S19L,
    #[serde(alias = "ANTMINER S19 PRO")]
    #[serde(alias = "s19pro")]
    S19Pro,
    #[serde(alias = "ANTMINER S19J")]
    #[serde(alias = "s19j")]
    S19j,
    #[serde(alias = "ANTMINER S19I")]
    #[serde(alias = "s19i")]
    S19i,
    #[serde(alias = "ANTMINER S19+")]
    #[serde(alias = "s19plus")]
    #[serde(alias = "s19+")]
    S19Plus,
    #[serde(alias = "ANTMINER S19J88NOPIC")]
    #[serde(alias = "s19j88nopic")]
    S19jNoPIC,
    #[serde(alias = "ANTMINER S19PRO+")]
    #[serde(alias = "s19proplus")]
    #[serde(alias = "s19pro+")]
    S19ProPlus,
    #[serde(alias = "ANTMINER S19J PRO")]
    #[serde(alias = "s19jpro")]
    S19jPro,
    #[serde(alias = "ANTMINER S19 XP")]
    #[serde(alias = "s19xp")]
    S19XP,
    #[serde(alias = "ANTMINER S19A")]
    #[serde(alias = "s19a")]
    S19a,
    #[serde(alias = "ANTMINER S19A PRO")]
    #[serde(alias = "s19apro")]
    S19aPro,
    #[serde(alias = "ANTMINER S19 HYDRO")]
    #[serde(alias = "s19hydro")]
    S19Hydro,
    #[serde(alias = "ANTMINER S19 PRO HYD.")]
    #[serde(alias = "s19prohydro")]
    S19ProHydro,
    #[serde(alias = "ANTMINER S19 PRO+ HYD.")]
    #[serde(alias = "s19proplushydro")]
    S19ProPlusHydro,
    #[serde(alias = "ANTMINER S19K PRO")]
    #[serde(alias = "s19kpro")]
    S19KPro,
    #[serde(alias = "ANTMINER S19J XP")]
    #[serde(alias = "s19jxp")]
    S19jXP,
    #[serde(alias = "ANTMINER T19")]
    #[serde(alias = "t19")]
    T19,
    #[serde(alias = "ANTMINER S21")]
    #[serde(alias = "ANTMINER BHB68601")]
    #[serde(alias = "ANTMINER BHB68606")]
    #[serde(alias = "s21")]
    #[serde(alias = "bhb68601")]
    #[serde(alias = "bhb68606")]
    S21,
    #[serde(alias = "ANTMINER S21 PRO")]
    #[serde(alias = "s21pro")]
    S21Pro,
    #[serde(alias = "ANTMINER S21+")]
    #[serde(alias = "s21plus")]
    #[serde(alias = "s21+")]
    S21Plus,
    #[serde(alias = "ANTMINER S21 HYD.")]
    #[serde(alias = "s21hydro")]
    S21Hydro,
    #[serde(alias = "ANTMINER S21+ HYD.")]
    #[serde(alias = "s21plushydro")]
    S21PlusHydro,
    #[serde(alias = "ANTMINER T21")]
    #[serde(alias = "t21")]
    T21,
}
