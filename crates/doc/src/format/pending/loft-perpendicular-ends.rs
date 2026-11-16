//! A `loft` feature can carry how it leaves its first section and arrives at its last:
//! `start_condition` and `end_condition` (`free` or `perpendicular`: perpendicular to that
//! section's sketch plane, towards the next section) and, for a perpendicular end, `start_weight`
//! and `end_weight` (unit-less expressions: how far each point runs on perpendicular before it
//! turns, as a share of its own distance to the next section; none is 1). Older designs have none
//! of them, so nothing in them is rewritten and their lofts keep both ends free. The last release
//! opens a design that has the fields with a warning about the newer format, builds the loft with
//! both ends free, and keeps the fields it cannot read when the design is saved.
