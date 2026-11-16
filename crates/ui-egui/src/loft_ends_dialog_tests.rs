//! The Loft dialog's start and end conditions (#7830): a perpendicular end shows its weight's
//! slider, the conditions and weights reach solid.loft, editing the loft brings them back and
//! keeps them, and dragging a slider changes the weight.
use crate::dialogs::Kind;
use crate::dialogs_loft::LoftEnds;
use crate::scenario::Harness;
use serde_json::json;
use solidcraft_engine::Sel;
use solidcraft_engine::doc::{Expr, FeatureKind, LoftEnd};

/// The square of side 10 `Low` at z = 0 and its copy `High` moved 6 along x at z = 8; their
/// sketch ids.
fn setup(h: &mut Harness) -> (u64, u64) {
    for (name, z, x) in [("Low", 0, 0), ("High", 8, 6)] {
        h.app.run("sketch.create", json!({"name": name, "plane": {"origin": [0, 0, z], "x_dir": [1, 0, 0], "y_dir": [0, 1, 0]}})).unwrap();
        h.app.run("sketch.rectangle.two_point", json!({"p0": [x, 0], "p1": [x + 10, 10]})).unwrap();
        h.app.run("sketch.finish", json!({})).unwrap();
    }
    let id = |h: &Harness, n: &str| h.app.session.doc.find_feature(n).unwrap().id;
    (id(h, "Low"), id(h, "High"))
}

/// The Loft dialog open with both squares picked.
fn open(h: &mut Harness) {
    let (low, high) = setup(h);
    h.step(&json!({"call":"ui.start","params":{"command":"solid.loft"}})).unwrap();
    h.frames(3);
    let d = h.app.dialog.as_mut().unwrap();
    d.inputs[0].items = vec![Sel::Profile { sketch: low, index: 0 }, Sel::Profile { sketch: high, index: 0 }];
}

fn ends(h: &mut Harness) -> &mut LoftEnds {
    let d = h.app.dialog.as_mut().unwrap();
    let Kind::Loft { ends, .. } = &mut d.kind else { panic!("the dialog is not Loft's") };
    ends
}

fn ok(h: &mut Harness) {
    h.frames(2);
    h.step(&json!({"widget":"OK"})).unwrap();
    assert!(h.app.dialog.is_none(), "{:?} {:?}", h.app.dialog.as_ref().map(|d| &d.error), h.app.session.log);
}

/// The last feature's end conditions and weights.
fn stored(h: &Harness, name: &str) -> ([LoftEnd; 2], [Option<Expr>; 2]) {
    match &h.app.session.doc.find_feature(name).unwrap().kind {
        FeatureKind::Loft { start_condition, end_condition, start_weight, end_weight, .. } => {
            ([*start_condition, *end_condition], [start_weight.clone(), end_weight.clone()])
        }
        k => panic!("{k:?}"),
    }
}

#[test]
fn a_perpendicular_end_shows_its_weight_and_is_kept_through_an_edit() {
    let mut h = Harness::new();
    open(&mut h);
    // Free ends have no weight to set.
    assert!(h.widget("Start Condition").is_some() && h.widget("End Condition").is_some());
    assert!(h.widget("Start Weight").is_none() && h.widget("End Weight").is_none());
    *ends(&mut h) = LoftEnds { condition: [1, 1], weight: ["1.2".into(), "0.6".into()] };
    h.frames(2);
    assert!(h.widget("Start Weight").is_some() && h.widget("End Weight").is_some());
    assert!(crate::scenario::rect_of("dialog:slider_track:Start Weight").is_some());
    ok(&mut h);
    let name = h.app.session.doc.features.last().unwrap().name.clone();
    let both = ([LoftEnd::Perpendicular; 2], [Some(Expr::new("1.2")), Some(Expr::new("0.6"))]);
    assert_eq!(stored(&h, &name), both);

    h.step(&json!({"call":"ui.editFeature","params":{"feature":name}})).unwrap();
    h.frames(8);
    assert_eq!(*ends(&mut h), LoftEnds { condition: [1, 1], weight: ["1.2".into(), "0.6".into()] });
    ok(&mut h);
    assert_eq!(stored(&h, &name), both);

    // Freeing an end in an edit drops its condition and its weight.
    h.step(&json!({"call":"ui.editFeature","params":{"feature":name}})).unwrap();
    h.frames(8);
    ends(&mut h).condition[0] = 0;
    ok(&mut h);
    assert_eq!(stored(&h, &name), ([LoftEnd::Free, LoftEnd::Perpendicular], [None, Some(Expr::new("0.6"))]));
}

#[test]
fn a_weight_left_at_its_default_is_not_stored_and_an_expression_is() {
    let mut h = Harness::new();
    open(&mut h);
    ends(&mut h).condition = [1, 0];
    ok(&mut h);
    let name = h.app.session.doc.features.last().unwrap().name.clone();
    assert_eq!(stored(&h, &name), ([LoftEnd::Perpendicular, LoftEnd::Free], [None, None]));

    // A weight that is not a plain number is an expression: its row is a field, not a slider.
    h.app.run("parameters.add", json!({"name": "pull", "expression": "1.5", "unit": ""})).unwrap();
    h.step(&json!({"call":"ui.editFeature","params":{"feature":name}})).unwrap();
    h.frames(8);
    ends(&mut h).weight[0] = "pull".into();
    h.frames(2);
    assert!(h.widget("Start Weight").is_some());
    assert!(crate::scenario::rect_of("dialog:slider_track:Start Weight").is_none());
    ok(&mut h);
    assert_eq!(stored(&h, &name).1, [Some(Expr::new("pull")), None]);
}

#[test]
fn dragging_a_weight_s_slider_changes_the_weight() {
    let mut h = Harness::new();
    open(&mut h);
    ends(&mut h).condition = [1, 1];
    h.frames(3);
    let track = crate::scenario::rect_of("dialog:slider_track:End Weight").unwrap();
    assert!(track.width() > 20.0, "end weight track: {track:?}");
    // From where the default weight 1 sits to three quarters of the way along the slider's
    // 0.1…3: the weight grows, stays in the slider's range and lands on one of its steps.
    let at = |share: f32| egui::pos2(track.left() + track.width() * share, track.center().y);
    let (start, end) = (at(0.9 / 2.9), at(0.75));
    h.step(&json!({"drag":[[start.x,start.y],[end.x,end.y]],"steps":8})).unwrap();
    h.frames(2);
    let weight: f64 = ends(&mut h).weight[1].parse().unwrap();
    assert!(weight > 1.5 && weight <= 3.0, "{weight}");
    assert!(((weight / 0.05).round() * 0.05 - weight).abs() < 1e-9, "{weight}");
    assert_eq!(ends(&mut h).weight[0], "1", "the other end's weight stays");
    ok(&mut h);
    let name = h.app.session.doc.features.last().unwrap().name.clone();
    let (conditions, weights) = stored(&h, &name);
    assert_eq!(conditions, [LoftEnd::Perpendicular; 2]);
    assert_eq!(weights, [None, Some(Expr::new(weight.to_string()))]);
}
