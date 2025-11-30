use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::Score;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub id: i32,
    pub file: String,
    pub name: String,
    pub desc: String,
    pub cond: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },

    Compare { stat: String, op: String, value: f64 },

    Range { stat: String, min: f64, max: f64 },

    BitEq { stat: String, mask: u32, equals: u32 },

    BitNe { stat: String, mask: u32 },
}

impl Condition {
    pub fn eval(&self, score: &Score) -> bool {
        // https://stackoverflow.com/questions/20737045/
        match self {
            Condition::And { conditions } => conditions.iter().all(|c| c.eval(score)),

            Condition::Or { conditions } => conditions.iter().any(|c| c.eval(score)),

            Condition::Compare { stat, op, value } => {
                let actual = score.get_ach_stat(stat);
                match op.as_str() {
                    ">" => actual > *value,
                    "<" => actual < *value,
                    "==" => (actual - value).abs() < f64::EPSILON,
                    "!=" => actual != *value,
                    ">=" => actual >= *value,
                    "<=" => actual <= *value,
                    _ => false,
                }
            },

            Condition::Range { stat, min, max } => {
                let actual = score.get_ach_stat(stat);
                actual >= *min && actual < *max
            },

            Condition::BitEq { stat, mask, equals } => {
                let actual = score.get_ach_stat(stat) as u32;
                (actual & mask) == *equals
            },

            Condition::BitNe { stat, mask } => {
                let actual = score.get_ach_stat(stat) as u32;
                (actual & mask) != 0
            },
        }
    }
}
