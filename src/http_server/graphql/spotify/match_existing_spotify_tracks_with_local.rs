use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::sync::Arc;

use crate::{database::Database, entities};
use color_eyre::eyre::{Context, OptionExt, Result};
use ollama_native::Ollama;
use regex::Regex;
use schemars::{JsonSchema, schema_for};
use sea_orm::ActiveModelTrait;
use sea_orm::{ColumnTrait, EntityTrait};
use sea_orm::{QueryFilter, Set};
use serde::{Deserialize, Serialize};
