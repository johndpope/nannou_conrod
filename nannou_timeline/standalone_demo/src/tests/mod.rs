//! Test modules for the timeline demo
//!
//! This module contains comprehensive tests for all major components
//! of the Flash-style timeline demo application.

#[cfg(test)]
mod simple_tests;

#[cfg(test)]
mod renderer_output_test;

#[cfg(test)]
mod visual_regression_test;

// Simple tests that can be run easily
pub use simple_tests::*;