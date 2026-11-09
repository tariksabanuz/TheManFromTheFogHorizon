// Test harness only: the modules in `mixins/the_man_from_the_fog/` are what ships. Each is built
// here with `include!` under the name Horizon's launcher gives it when it injects it into
// `game.rs`, so `super::native_the_man_from_the_fog_*` paths work in tests and in the game.
#![allow(dead_code)]

macro_rules! module {
    ($name:ident, $file:literal) => {
        pub mod $name {
            include!(concat!(env!("CARGO_MANIFEST_DIR"), "/mixins/the_man_from_the_fog/", $file));
        }
    };
}

module!(native_the_man_from_the_fog_config, "config.rs");
module!(native_the_man_from_the_fog_fog, "fog.rs");
module!(native_the_man_from_the_fog_model, "model.rs");
module!(native_the_man_from_the_fog_runtime, "runtime.rs");
module!(native_the_man_from_the_fog_scare, "scare.rs");
module!(native_the_man_from_the_fog_state, "state.rs");
module!(native_the_man_from_the_fog_view, "view.rs");
