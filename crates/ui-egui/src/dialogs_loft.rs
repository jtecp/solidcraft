//! How a Loft leaves its first section and arrives at its last, in its dialog: a condition for
//! each end (free, or perpendicular to the section's sketch plane) and, for a perpendicular end,
//! its weight on a slider. The weight is how far each point of the section runs on perpendicular
//! before it turns, as a share of its own distance to the next section.

use std::ops::RangeInclusive;

use serde_json::{Value, json};
use solidcraft_engine::doc::{Expr, LoftEnd};

use crate::ui_kit::{Slider, choice_row, expr_row_positive, row, slider};

/// The conditions, in combo order: free, perpendicular.
const CONDITIONS: [&str; 2] = ["Free", "Perpendicular"];
/// The condition that takes a weight.
const PERPENDICULAR: usize = 1;
/// A weight field that means the weight a perpendicular end has when none is given.
const NO_WEIGHT: &str = "1";
/// The weights the slider reaches. Up to 3 at both ends a wall between sections on parallel
/// planes stays between the planes; a weight outside the range is typed, as an expression is.
const SLIDER: RangeInclusive<f64> = 0.1..=3.0;
/// The slider's step.
const STEP: f64 = 0.05;
/// Each end's rows and parameters: the condition's label, the weight's label, the condition
/// combo's id, the condition's parameter and the weight's.
const ENDS: [(&str, &str, &str, &str, &str); 2] = [
    ("Start Condition", "Start Weight", "lf_start", "start_condition", "start_weight"),
    ("End Condition", "End Weight", "lf_end", "end_condition", "end_weight"),
];

/// A loft dialog's end conditions: the start's, then the end's.
#[derive(Clone, Debug, PartialEq)]
pub struct LoftEnds {
    /// An index into [`CONDITIONS`] for each end.
    pub condition: [usize; 2],
    /// Each end's weight as its field's text: a number the slider moves, or an expression.
    pub weight: [String; 2],
}

impl Default for LoftEnds {
    fn default() -> Self {
        LoftEnds { condition: [0, 0], weight: [NO_WEIGHT.into(), NO_WEIGHT.into()] }
    }
}

impl LoftEnds {
    /// A feature's stored conditions and weights (none: the default weight), for editing it.
    pub fn of(start: (LoftEnd, &Option<Expr>), end: (LoftEnd, &Option<Expr>)) -> Self {
        let condition = |(c, _): (LoftEnd, &Option<Expr>)| usize::from(c == LoftEnd::Perpendicular);
        let weight = |(_, w): (LoftEnd, &Option<Expr>)| w.as_ref().map_or_else(|| NO_WEIGHT.into(), |w| w.to_string());
        LoftEnds { condition: [condition(start), condition(end)], weight: [weight(start), weight(end)] }
    }
}

/// A slider's value as the weight field's text: two decimals at most, without trailing zeros.
fn number_text(v: f64) -> String {
    let text = format!("{v:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A weight's row: a slider while the field holds a plain number in the slider's range, the
/// expression field otherwise. True when Enter was pressed in the field.
fn weight_row(ui: &mut egui::Ui, label: &str, text: &mut String) -> bool {
    let Some(mut v) = text.trim().parse::<f64>().ok().filter(|v| SLIDER.contains(v)) else {
        return expr_row_positive(ui, label, text);
    };
    row(ui, label, |ui| {
        if slider(ui, &mut v, SLIDER, Slider { step: Some(STEP), ..Default::default() }).changed() {
            *text = number_text(v);
        }
    });
    false
}

/// The end rows: each end's condition and, when it is perpendicular, its weight. True when Enter
/// was pressed in a field.
pub fn rows(ui: &mut egui::Ui, o: &mut LoftEnds) -> bool {
    let mut enter = false;
    for ((condition, weight), (condition_label, weight_label, id, _, _)) in o.condition.iter_mut().zip(&mut o.weight).zip(ENDS) {
        choice_row(ui, condition_label, id, condition, &CONDITIONS);
        if *condition == PERPENDICULAR {
            enter |= weight_row(ui, weight_label, weight);
        }
    }
    enter
}

/// The end conditions as `solid.loft` parameters, added to `p`: a perpendicular end's condition,
/// and its weight when it is not the default. A free end adds nothing.
pub fn params(o: &LoftEnds, p: &mut Value) {
    for ((condition, weight), (_, _, _, condition_key, weight_key)) in o.condition.iter().zip(&o.weight).zip(ENDS) {
        if *condition != PERPENDICULAR {
            continue;
        }
        p[condition_key] = json!("perpendicular");
        let weight = weight.trim();
        if !weight.is_empty() && weight != NO_WEIGHT {
            p[weight_key] = json!(weight);
        }
    }
}
