#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate lazy_static;
extern crate serde_json;
extern crate netcdf;
extern crate ndarray;
extern crate glob;
extern crate serde;
extern crate env_logger;


pub mod json_structs;
pub mod elevation;

// Local mods
#[cfg(test)]
mod tests;