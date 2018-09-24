use roots::{find_roots_cubic, Roots};

///
/// Solves for t in a single dimension for a bezier curve (finds the point(s) where the basis
/// function evaluates to p)
/// 
pub fn solve_basis_for_t(w1: f64, w2: f64, w3: f64, w4: f64, p: f64) -> Vec<f64> {
    // Compute the coefficients for the cubic bezier function
    let d = w1-p;
    let c = 3.0*(w2-w1);
    let b = 3.0*(w3-w2)-c;
    let a = w4-w1-c-b;

    // Solve for p
    let roots = find_roots_cubic(a, b, c, d);
    let mut roots = match roots {
        Roots::No(_)    => vec![],
        Roots::One(r)   => r.to_vec(),
        Roots::Two(r)   => r.to_vec(),
        Roots::Three(r) => r.to_vec(),
        Roots::Four(r)  => r.to_vec()
    };

    // Clip to 0/1 for small ranges outside
    for mut root in roots.iter_mut() {
        if *root < 0.0 && *root > -0.001 { *root = 0.0 }
        if *root > 1.0 && *root < 1.001 { *root = 1.0 }
    }

    // Remove any roots outside the range of the function
    roots.retain(|r| r >= &0.0 && r <= &1.0);

    // Return the roots
    roots
}
