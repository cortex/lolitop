pub mod camera;
pub mod cpu;
pub mod light;
pub mod metrics;
pub mod model;
pub mod state;
pub mod text;
pub mod ui;
pub mod window;

#[macro_export]
macro_rules! lines {
    ($($x:expr),*) => {
        concat!($( $x, "\n" ),*)
    };
}
