// NOTE: axum Response is big by nature and every route handler returns
// Result<_, Response> — boxing it everywhere would churn the whole tree
// for zero runtime gain, so the lint stays off.
#![allow(clippy::result_large_err)]

pub mod config;
pub mod constants;
pub mod dto;
pub mod geoloc;
pub mod infrastructure;
pub mod models;
pub mod repository;
pub mod routes;
pub mod state;
pub mod usecases;
pub mod utils;
