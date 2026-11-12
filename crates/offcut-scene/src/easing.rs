//! Easing curves: how far a motion has come, given how much of its time has passed.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseOutCubic,
    /// Goes a little past the end and comes back.
    BackOut,
}

const BACK_OVERSHOOT: f32 = 1.70158;

/// `x` is the share of the time that has passed, read as within `0.0..=1.0`.
/// The result is 0.0 at 0.0 and 1.0 at 1.0; `BackOut` passes 1.0 on the way.
pub fn ease(e: Easing, x: f32) -> f32 {
    let x = if x.is_nan() { 0.0 } else { x.clamp(0.0, 1.0) };
    match e {
        Easing::Linear => x,
        Easing::EaseOutCubic => 1.0 - (1.0 - x).powi(3),
        Easing::BackOut => {
            let back = x - 1.0;
            1.0 + (BACK_OVERSHOOT + 1.0) * back.powi(3) + BACK_OVERSHOOT * back.powi(2)
        }
    }
}

// --- V6.2: cubic bezier curves (v6.2implementation D-475, TS §47.2) -----------
//
// What every V6.2 motion uses: the camera drift, the zooms and the entries.
// `Easing` and `ease()` above are unchanged, so the styles that use them keep
// their frames.

/// A CSS-style cubic bezier from (0, 0) to (1, 1) through the control points
/// `(x1, y1)` and `(x2, y2)`. With `x1` and `x2` in `0.0..=1.0` the time
/// coordinate rises, so each time has one value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// Starts slowly and settles slowly: the camera drift's curve.
pub const STANDARD: CubicBezier = CubicBezier {
    x1: 0.4,
    y1: 0.0,
    x2: 0.2,
    y2: 1.0,
};
/// Starts at speed and settles slowly.
pub const OUT: CubicBezier = CubicBezier {
    x1: 0.0,
    y1: 0.0,
    x2: 0.2,
    y2: 1.0,
};
/// Slow at both ends, symmetric about the middle.
pub const IN_OUT: CubicBezier = CubicBezier {
    x1: 0.65,
    y1: 0.0,
    x2: 0.35,
    y2: 1.0,
};

/// The time coordinate is solved to this distance.
const SOLVE_EPSILON: f64 = 1e-7;
const NEWTON_STEPS: u32 = 8;
const BISECTION_STEPS: u32 = 60;

/// One coordinate of the curve at the parameter `u`, its control values
/// `p1` and `p2` (the ends being 0 and 1).
fn coord(p1: f64, p2: f64, u: f64) -> f64 {
    let v = 1.0 - u;
    3.0 * v * v * u * p1 + 3.0 * v * u * u * p2 + u * u * u
}

/// Its derivative in `u`.
fn slope(p1: f64, p2: f64, u: f64) -> f64 {
    let v = 1.0 - u;
    3.0 * v * v * p1 + 6.0 * v * u * (p2 - p1) + 3.0 * u * u * (1.0 - p2)
}

/// The parameter whose time coordinate is `x`, by Newton's method from
/// `u = x`: `None` when a step meets a flat slope, leaves `0..=1` or has
/// not converged.
fn newton(c: CubicBezier, x: f64) -> Option<f64> {
    let (x1, x2) = (f64::from(c.x1), f64::from(c.x2));
    let mut u = x;
    for _ in 0..NEWTON_STEPS {
        let error = coord(x1, x2, u) - x;
        if error.abs() < SOLVE_EPSILON {
            return Some(u);
        }
        let d = slope(x1, x2, u);
        if d.abs() < 1e-6 {
            return None;
        }
        u -= error / d;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }
    }
    (coord(x1, x2, u) - x).abs().lt(&SOLVE_EPSILON).then_some(u)
}

/// The same parameter by bisection: always found, as the time coordinate rises.
fn bisection(c: CubicBezier, x: f64) -> f64 {
    let (x1, x2) = (f64::from(c.x1), f64::from(c.x2));
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..BISECTION_STEPS {
        let mid = (lo + hi) / 2.0;
        if coord(x1, x2, mid) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// How far a motion on `c` has come when the share `t` of its time has
/// passed, read as within `0.0..=1.0` (NaN as 0). Exactly 0.0 at 0.0 and
/// 1.0 at 1.0. Newton's method, with bisection where it fails.
pub fn bezier(c: CubicBezier, t: f32) -> f32 {
    let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let x = f64::from(t);
    let u = newton(c, x).unwrap_or_else(|| bisection(c, x));
    coord(f64::from(c.y1), f64::from(c.y2), u) as f32
}

#[cfg(test)]
mod bezier_tests {
    use super::*;

    const CURVES: [(&str, CubicBezier); 3] =
        [("STANDARD", STANDARD), ("OUT", OUT), ("IN_OUT", IN_OUT)];

    #[test]
    fn the_ends_are_exact_and_outside_is_held() {
        for (name, c) in CURVES {
            assert_eq!(bezier(c, 0.0), 0.0, "{name}");
            assert_eq!(bezier(c, 1.0), 1.0, "{name}");
            assert_eq!(bezier(c, -0.5), 0.0, "{name}");
            assert_eq!(bezier(c, 2.0), 1.0, "{name}");
            assert_eq!(bezier(c, f32::NAN), 0.0, "{name}");
        }
    }

    #[test]
    fn each_curve_rises_over_1000_points() {
        for (name, c) in CURVES {
            let mut before = 0.0;
            for i in 1..=1000u16 {
                let now = bezier(c, f32::from(i) / 1000.0);
                assert!(now >= before, "{name} falls at {i}: {before} then {now}");
                assert!((0.0..=1.0).contains(&now), "{name} at {i}: {now}");
                before = now;
            }
        }
    }

    /// Values computed apart from this code (P199): bisection to 1e-13 in
    /// Python on the same control points, rounded to six places.
    #[test]
    fn known_values_to_1e_4() {
        let known: [(CubicBezier, [f32; 5]); 3] = [
            (STANDARD, [0.025863, 0.236587, 0.775561, 0.959368, 0.994354]),
            (OUT, [0.303848, 0.577573, 0.839245, 0.964216, 0.994601]),
            (IN_OUT, [0.008943, 0.070797, 0.5, 0.929203, 0.991057]),
        ];
        for (c, values) in known {
            for (t, want) in [0.1, 0.25, 0.5, 0.75, 0.9].into_iter().zip(values) {
                let got = bezier(c, t);
                assert!((got - want).abs() < 1e-4, "{c:?} at {t}: {got}, not {want}");
            }
        }
    }

    #[test]
    fn newton_and_bisection_agree() {
        for (name, c) in CURVES {
            let mut newton_used = 0;
            for i in 1..1000u16 {
                let x = f64::from(i) / 1000.0;
                let by_bisection = bisection(c, x);
                if let Some(by_newton) = newton(c, x) {
                    newton_used += 1;
                    let (y1, y2) = (f64::from(c.y1), f64::from(c.y2));
                    let gap = (coord(y1, y2, by_newton) - coord(y1, y2, by_bisection)).abs();
                    assert!(gap < 1e-5, "{name} at {x}: {gap}");
                }
            }
            // Newton's method answers nearly everywhere; bisection is the fallback.
            assert!(newton_used > 900, "{name}: Newton {newton_used} of 999");
        }
    }

    #[test]
    fn in_out_is_symmetric() {
        for i in 0..=100u8 {
            let t = f32::from(i) / 100.0;
            let (a, b) = (bezier(IN_OUT, t), 1.0 - bezier(IN_OUT, 1.0 - t));
            assert!((a - b).abs() < 1e-5, "{t}: {a} and {b}");
        }
    }
}
