//! A loft's perpendicular start and end through solid.loft (#7830): the body against its closed
//! forms, either order of the sections, editing, a weight from a parameter, saving, naming, and
//! hostile values.
//!
//! The closed forms: a row of the loft runs from a point P of one section to the point Q it is
//! joined to in the next, c = |Q - P| apart. With both ends perpendicular (weights a and b) it is
//! the cubic Bezier P, P + (a c / 3) n, Q - (b c / 3) m, Q, n and m the sections' normals along
//! the loft; with one perpendicular end (weight a) the quadratic P, P + (a c / 2) n, Q.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::f64::consts::PI;

use serde_json::{Value, json};
use solidcraft_geom::Vec3Geom;

use crate::Session;

fn run(s: &mut Session, cmd: &str, p: Value) -> Value {
    s.execute(cmd, &p).unwrap_or_else(|e| panic!("{cmd}: {e}"))
}

/// A sketch `name` on the plane through `origin` with the axes `x` and `y`, holding one `cmd`.
fn sketch(s: &mut Session, name: &str, origin: [i32; 3], x: [i32; 3], y: [i32; 3], draw: &[(&str, Value)]) {
    run(s, "sketch.create", json!({"name": name, "plane": {"origin": origin, "x_dir": x, "y_dir": y}}));
    for (cmd, p) in draw {
        run(s, cmd, p.clone());
    }
    run(s, "sketch.finish", json!({}));
}

/// The square of side 10 `Low` at z = 0 and its copy `High` moved 6 along x at z = 8 (every
/// corner's row 10 long); the rings `RingLow` and `RingHigh` (radius 10 round a hole of 5) at
/// z = 0 and 40; the squares `Floor` (z = 0, about the origin) and `Wall` (in the plane x = 20,
/// about (20, 0, 20)), at a right angle to each other; `Beside`, `Low`'s copy beside it in its
/// own plane; and in `Apex` a point above `Low`.
fn setup() -> Session {
    let mut s = Session::default();
    let (x, y, z) = ([1, 0, 0], [0, 1, 0], [0, 0, 1]);
    let rect = |p0: [i32; 2], p1: [i32; 2]| ("sketch.rectangle.two_point", json!({"p0": p0, "p1": p1}));
    let circle = |r: i32| ("sketch.circle.center", json!({"center": [0, 0], "radius": r}));
    sketch(&mut s, "Low", [0, 0, 0], x, y, &[rect([0, 0], [10, 10])]);
    sketch(&mut s, "High", [0, 0, 8], x, y, &[rect([6, 0], [16, 10])]);
    sketch(&mut s, "RingLow", [0, 0, 0], x, y, &[circle(10), circle(5)]);
    sketch(&mut s, "RingHigh", [0, 0, 40], x, y, &[circle(10), circle(5)]);
    sketch(&mut s, "Floor", [0, 0, 0], x, y, &[rect([-5, -5], [5, 5])]);
    sketch(&mut s, "Wall", [20, 0, 20], y, z, &[rect([-5, -5], [5, 5])]);
    sketch(&mut s, "Beside", [0, 0, 0], x, y, &[rect([30, 0], [40, 10])]);
    sketch(&mut s, "Apex", [0, 0, 20], x, y, &[("sketch.point", json!({"point": [5, 5], "id": "tip"}))]);
    s
}

fn measure(s: &Session, body: &str) -> solidcraft_kernel::BodyMeasure {
    solidcraft_kernel::measure(&s.world_state().unwrap().body(body).unwrap().body).unwrap()
}

/// solid.loft through the sketches `sections` as a new body `body`, with `extra` parameters.
fn loft(s: &mut Session, sections: [&str; 2], body: &str, extra: Value) -> Result<Value, String> {
    let mut p = json!({"sections": [{"sketch": sections[0]}, {"sketch": sections[1]}], "operation": "new", "body_name": body});
    p.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    s.execute("solid.loft", &p).map_err(|e| e.to_string())
}

fn face_names(s: &mut Session, body: &str) -> Vec<String> {
    let f = run(s, "model.faces", json!({ "body": body }));
    let mut n: Vec<String> = f["faces"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x["name"].as_str())
        .map(|x| x.split_once(':').map_or(x, |(_, rest)| rest).to_string())
        .collect();
    n.sort();
    n
}

/// `Low` to `High` with the weights 1.2 and 0.6: a corner's row has z = 0, 4, 6, 8 and
/// x = 0, 0, 6, 6 for its Bezier points, z rises all the way (z' = 12 - 12 v + 6 v²), so every
/// level cuts the square moved by x: the volume is the sheared box's, 800, and the centre's x is
/// 5 + ∫ x z' dv / 8 with x = 6 (3 v² - 2 v³): 5 + 6 (12 - 15 + 8.4 - 2) / 8 = 7.55. The plain
/// loft's centre is at x = 8. The sections given the other way round, with the weights swapped,
/// are the same body, and its faces keep the plain loft's names.
#[test]
fn perpendicular_ends_shape_a_loft_to_its_closed_form_in_either_order() {
    let mut s = setup();
    loft(&mut s, ["Low", "High"], "Plain", json!({})).unwrap();
    let plain = measure(&s, "Plain");
    assert!((plain.volume - 800.0).abs() < 1e-6 && (plain.centroid.x - 8.0).abs() < 1e-6, "{plain:?}");

    let both = json!({"start_condition": "perpendicular", "end_condition": "perpendicular", "start_weight": 1.2, "end_weight": 0.6});
    loft(&mut s, ["Low", "High"], "Curved", both).unwrap();
    let m = measure(&s, "Curved");
    assert!((m.volume - 800.0).abs() < 1e-5, "volume {}", m.volume);
    assert!((m.centroid.x - 7.55).abs() < 1e-6 && (m.centroid.y - 5.0).abs() < 1e-6 && (m.centroid.z - 4.0).abs() < 1e-6, "{:?}", m.centroid);
    assert_eq!(face_names(&mut s, "Plain"), face_names(&mut s, "Curved"));

    let turned = json!({"start_condition": "perpendicular", "end_condition": "perpendicular", "start_weight": 0.6, "end_weight": 1.2});
    loft(&mut s, ["High", "Low"], "Turned", turned).unwrap();
    let t = measure(&s, "Turned");
    assert!((t.volume - m.volume).abs() < 1e-9 * m.volume && t.centroid.dist(m.centroid) < 1e-9, "{:?} vs {:?}", t.centroid, m.centroid);
}

/// The conditions and weights are the feature's own: saved with it, and changed by an edit. A
/// start weight of 1.8 (end 0.6) makes z = 0, 6, 6, 8 (z' = 18 (1 - v)² + 6 v²), the centre's x
/// 5 + 6 (18 - 36 + 28.8 - 8) / 8 = 7.1. With the start freed, the end alone makes the
/// quadratic z = 0, 5, 8 and x = 0, 6, 6 (z' = 10 - 4 v, x = 12 v - 6 v²): 5 + 30 / 8 = 8.75.
/// A weight left out is 1: the start alone then makes z = 0, 5, 8 and x = 0, 0, 6
/// (z' = 10 - 4 v, x = 6 v²), 5 + (20 - 6) / 8 = 6.75.
#[test]
fn a_loft_s_ends_are_saved_and_edited() {
    let mut s = setup();
    let both = json!({"start_condition": "perpendicular", "end_condition": "perpendicular", "start_weight": 1.2, "end_weight": 0.6, "name": "Bent"});
    loft(&mut s, ["Low", "High"], "Body", both).unwrap();
    let x = |s: &Session| measure(s, "Body").centroid.x;
    assert!((x(&s) - 7.55).abs() < 1e-6, "{}", x(&s));

    let reopened = Session::new(solidcraft_doc::Document::from_json(&s.doc.to_json()).unwrap());
    assert!((x(&reopened) - x(&s)).abs() < 1e-12);

    run(&mut s, "timeline.edit", json!({"feature": "Bent", "set": {"start_weight": 1.8}}));
    assert!((x(&s) - 7.1).abs() < 1e-6, "{}", x(&s));
    run(&mut s, "timeline.edit", json!({"feature": "Bent", "set": {"start_condition": "free"}}));
    assert!((x(&s) - 8.75).abs() < 1e-6, "{}", x(&s));
    run(
        &mut s,
        "timeline.edit",
        json!({"feature": "Bent", "set": {"start_condition": "perpendicular", "start_weight": null, "end_condition": "free"}}),
    );
    assert!((x(&s) - 6.75).abs() < 1e-6, "{}", x(&s));
    // Out of the kernel's range, an edit is refused as the command refuses it.
    assert!(s.execute("timeline.edit", &json!({"feature": "Bent", "set": {"start_weight": 50}})).is_err());
    assert!((x(&s) - 6.75).abs() < 1e-6, "{}", x(&s));
}

/// A weight is an expression: a parameter drives it.
#[test]
fn a_loft_end_s_weight_follows_a_parameter() {
    let mut s = setup();
    run(&mut s, "parameters.add", json!({"name": "pull", "expression": "1.2", "unit": ""}));
    let both = json!({"start_condition": "perpendicular", "end_condition": "perpendicular", "start_weight": "pull", "end_weight": "pull / 2"});
    loft(&mut s, ["Low", "High"], "Body", both).unwrap();
    assert!((measure(&s, "Body").centroid.x - 7.55).abs() < 1e-6);
    // 1.8 and 0.9: z = 0, 6, 5, 8, z' = 18 (1 - v)² - 6 v (1 - v) + 9 v² = 18 - 42 v + 33 v²
    // (above 0 all the way), and ∫ (3 v² - 2 v³) z' dv = (18 - 31.5 + 19.8) - (9 - 16.8 + 11)
    // = 3.1: the centre's x is 5 + 6 * 3.1 / 8.
    run(&mut s, "parameters.change", json!({"name": "pull", "expression": "1.8"}));
    let x = measure(&s, "Body").centroid.x;
    assert!((x - 7.325).abs() < 1e-6, "{x}");
}

/// Sections with a hole: the outline and the hole leave perpendicular alike, so between two
/// equal rings with weight 1 the walls are the straight ones and the loft is the tube.
#[test]
fn perpendicular_ends_keep_a_section_s_hole() {
    let mut s = setup();
    let ring = |name: &str| json!({"sketch": name, "profiles": [{"point": [7.5, 0]}]});
    let p = json!({"sections": [ring("RingLow"), ring("RingHigh")], "operation": "new", "body_name": "Tube",
        "start_condition": "perpendicular", "end_condition": "perpendicular"});
    run(&mut s, "solid.loft", p);
    let volume = PI * (100.0 - 25.0) * 40.0;
    let m = measure(&s, "Tube");
    assert!((m.volume - volume).abs() < 1e-6 * volume, "{} vs {volume}", m.volume);
}

/// Sections at a right angle, `Floor` and `Wall`: each row leaves the floor straight up and
/// arrives at the wall square to it, the rows near the inside of the bend shorter than those on
/// its outside. Every row's legs are the same share of its own length, so all of them stay
/// between the floor and the wall's far edge up to the same weight, 3 / √2 (a row runs as far
/// along x as along z, its chord √2 times that): no row runs under the floor or behind the wall,
/// however near the inside of the bend the sections come. The bend is its own mirror image in
/// the plane x + z = 20, so with equal weights the centre lies in it.
#[test]
fn a_right_angle_bend_stays_inside_its_sections_planes() {
    let mut s = setup();
    for (i, weight) in [0.5, 1.0, 2.0].into_iter().enumerate() {
        let body = format!("Bend{i}");
        let both = json!({"start_condition": "perpendicular", "end_condition": "perpendicular", "start_weight": weight, "end_weight": weight});
        loft(&mut s, ["Floor", "Wall"], &body, both).unwrap();
        let m = measure(&s, &body);
        assert!(m.volume > 0.0 && (m.centroid.x + m.centroid.z - 20.0).abs() < 1e-6 && m.centroid.y.abs() < 1e-6, "{weight}: {:?}", m.centroid);
        assert!(m.bbox.min.z > -1e-6 && m.bbox.max.x < 20.0 + 1e-6, "{weight}: {:?}", m.bbox);
        assert!(m.bbox.min.x > -5.0 - 1e-6 && m.bbox.max.z < 25.0 + 1e-6, "{weight}: {:?}", m.bbox);
    }
}

/// Hostile end conditions are refused with an error and leave the design as it was.
#[test]
fn hostile_loft_ends_are_refused() {
    let mut s = setup();
    let before = s.doc.features.len();
    for bad in [
        json!({"start_condition": "wobble"}),
        json!({"end_condition": 1}),
        json!({"start_condition": ["perpendicular"]}),
        // A weight belongs to a perpendicular end.
        json!({"start_weight": 1}),
        json!({"start_condition": "free", "start_weight": 1}),
        json!({"start_condition": "perpendicular", "end_weight": 1}),
        // Weights the kernel does not take.
        json!({"start_condition": "perpendicular", "start_weight": 0}),
        json!({"start_condition": "perpendicular", "start_weight": -1}),
        json!({"end_condition": "perpendicular", "end_weight": 10.5}),
        json!({"end_condition": "perpendicular", "end_weight": "nope"}),
        json!({"end_condition": "perpendicular", "end_weight": "5 mm"}),
        json!({"end_condition": "perpendicular", "end_weight": [1]}),
    ] {
        assert!(loft(&mut s, ["Low", "High"], "Bad", bad.clone()).is_err(), "{bad}");
    }
    let perpendicular = json!({"start_condition": "perpendicular"});
    // No side to leave to: the next section in the first one's own plane.
    assert!(loft(&mut s, ["Low", "Beside"], "Bad", perpendicular.clone()).is_err());
    // With what a perpendicular end does not build yet: an apex, and guide rails.
    let apex = json!({"sections": [{"sketch": "Low"}, {"sketch": "Apex", "point": "tip"}], "start_condition": "perpendicular"});
    let refused = s.execute("solid.loft", &apex).unwrap_err().to_string();
    assert!(refused.contains("not supported yet"), "{refused}");
    sketch(&mut s, "Rail", [0, 0, 0], [1, 0, 0], [0, 0, 1], &[("sketch.line", json!({"points": [[0, 0], [6, 8]], "ids": ["r"]}))]);
    let before = before + 1;
    let railed = json!({"rails": [{"sketch": "Rail", "curves": ["r"]}], "end_condition": "perpendicular"});
    let refused = loft(&mut s, ["Low", "High"], "Bad", railed).unwrap_err();
    assert!(refused.contains("not supported yet"), "{refused}");
    assert_eq!(s.doc.features.len(), before, "nothing was added");
}
