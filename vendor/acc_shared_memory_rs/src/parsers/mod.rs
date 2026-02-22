pub mod physics_parser;
pub mod graphics_parser;
pub mod statics_parser;

pub use physics_parser::parse_physics_map;
pub use graphics_parser::parse_graphics_map;
pub use statics_parser::parse_statics_map;