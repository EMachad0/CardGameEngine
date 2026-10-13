macro_rules! def_ids {
    ($($name:ident = $id:literal),* $(,)?) => {
        pub mod static_card_definition {
            use crate::cards::definition::DefId;

            $(
                pub const $name: DefId = DefId::new($id);
            )*

            pub(in crate::cards) const ALL_DEF_ID: &[DefId] = &[
            $(
                $name,
            )*
            ];
        }
    };
}

def_ids!(
    CAPTAIN = "base.captain.v0",
    BLAST = "base.blast.v0",
    RECRUIT = "base.recruit.v0",
    GIANT = "base.giant.v0",
    SPARK = "base.spark.v0",
    BOLT = "base.bolt.v0",
    FORAGE = "base.forage.v0",
    WILD_BOLT = "base.wild_bolt.v0",
    HERO = "base.hero.v0",
    SQUIRE = "base.squire.v0",
    BARRACKS = "base.barracks.v0",
    ZAP = "base.zap.v0",
    STRAY_SHOT = "base.stray_shot.v0",
    PING = "base.ping.v0",
    SHOVE = "base.shove.v0",
    TWIN_SHOT = "base.twin_shot.v0",
    CROSSFIRE = "base.crossfire.v0",
    MEDIC = "base.medic.v0",
    MOURNER = "base.mourner.v0",
    DRILLMASTER = "base.drillmaster.v0",
    RALLY_BANNER = "base.rally_banner.v0",
    BOMBER = "base.bomber.v0",
    GRAVEDIGGER = "base.gravedigger.v0",
    SWEEP = "base.sweep.v0",
    HOARDER = "base.hoarder.v0",
    DRUMMER = "base.drummer.v0",
);
