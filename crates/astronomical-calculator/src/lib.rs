// #![no_std]

pub(crate) mod spa2;

pub(crate) mod tables;

pub(crate) mod types;

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod unsafe_spa;
