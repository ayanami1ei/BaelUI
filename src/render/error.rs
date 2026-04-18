#![allow(unused)]

use std::fmt;

use thiserror::Error;
use winit::error::EventLoopError;

#[derive(Debug, Error)]
pub(crate) enum RenderError {
    WindowError(#[from] EventLoopError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::WindowError(e) => write!(f, "Window error: {}", e),
        }
    }
}