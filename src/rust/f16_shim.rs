// src/f16_shim.rs  (mod it in from main/lib)
#![allow(non_snake_case)]
#![allow(non_snake_case)]

#[unsafe(no_mangle)]
pub extern "C" fn __gnu_h2f_ieee(_a: u16) -> f32 {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn __gnu_f2h_ieee(_a: f32) -> u16 {
    loop {}
}