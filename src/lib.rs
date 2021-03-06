#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![deny(missing_docs)]
//! Defines 3-space and implements the boolean GJK (BGJK) algorithm
//! for intersection testing.
use std::ops::{Neg, Sub};

/// Vector for use in the `bgjk` function
///
/// Uses cartesian spatial dimensions in the order
/// x, y, z.
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl Eq for Vec3 {}

impl PartialEq for Vec3 {
	fn eq(&self, other: &Vec3) -> bool {
		self.0 == other.0 && self.1 == other.1 && self.2 == other.2
	}
}

impl Sub for Vec3 {
	type Output = Vec3;
	fn sub(self, right: Vec3) -> Self::Output {
		Vec3(self.0 - right.0, self.1 - right.1, self.2 - right.2)
	}
}

impl Vec3 {
	fn dot(&self, right: Vec3) -> f32 {
		self.0 * right.0 + self.1 * right.1 + self.2 * right.2
	}

	fn ones() -> Vec3 {
		Vec3(1.0, 1.0, 1.0)
	}
}

impl Neg for Vec3 {
	type Output = Vec3;
	fn neg(self) -> Self::Output {
		Vec3(-self.0, -self.1, -self.2)
	}
}

/// The BGJK algorithm
///
/// The Boolean-GJK algorithm gives us the answer to the question:
/// "do these convex hulls intersect?"
/// This algorithm takes two hulls. The ordering of the points is not
/// important. All points are assumed to be on the surface of the hull.
/// Having interior points should not affect the qualitative result of
/// the algorithm, but may cause slight (very minor) degradation in
/// performance. The algorithm is O(n+m), where n and m are the amount
/// of points in hull1 and hull2 respectively.
pub fn bgjk(hull1: &[Vec3], hull2: &[Vec3]) -> bool {
	let mut sp = Vec3::ones();
	let mut dp = Vec3::default();
	let (mut ap, mut bp, mut cp);

	cp = support(hull1, hull2, sp);
	sp = -cp;
	bp = support(hull1, hull2, sp);
	if bp.dot(sp) < 0.0 {
		return false;
	}
	sp = dcross3(cp - bp, -bp);
	let mut w = 2;

	loop {
		ap = support(hull1, hull2, sp);
		if ap.dot(sp) < 0.0 {
			return false;
		} else if simplex(&mut ap, &mut bp, &mut cp, &mut dp, &mut sp, &mut w) {
			return true;
		}
	}
}

// Todo clean up signature, this has to be fixed, sending 6 ptrs...
fn simplex(ap: &mut Vec3,
           bp: &mut Vec3,
           cp: &mut Vec3,
           dp: &mut Vec3,
           sp: &mut Vec3,
           w: &mut i32)
           -> bool {
	let ao = -*ap;
	let mut ab = *bp - *ap;
	let mut ac = *cp - *ap;
	let mut abc = cross(ab, ac);
	match *w {
		2 => {
			let ab_abc = cross(ab, abc);
			if ab_abc.dot(ao) > 0.0 {
				*cp = *bp;
				*bp = *ap;
				*sp = dcross3(ab, ao);
			} else {
				let abc_ac = cross(abc, ac);
				if abc_ac.dot(ao) > 0.0 {
					*bp = *ap;
					*sp = dcross3(ac, ao);
				} else {
					if abc.dot(ao) > 0.0 {
						*dp = *cp;
						*cp = *bp;
						*bp = *ap;
						*sp = abc;
					} else {
						*dp = *bp;
						*bp = *ap;
						*sp = -abc;
					}
					*w = 3;
				}
			}
			false
		}
		3 => {
			macro_rules! check_tetrahedron {
				() => { check_tetra(Tetra(ap, bp, cp, dp), sp, w, ao, ab, ac, abc); };
			};
			if abc.dot(ao) > 0.0 {
				check_tetrahedron![];;
				false
			} else {
				let ad = *dp - *ap;
				let acd = cross(ac, ad);
				if acd.dot(ao) > 0.0 {
					*bp = *cp;
					*cp = *dp;
					ab = ac;
					ac = ad;
					abc = acd;
					check_tetrahedron![];;
					false
				} else {
					let adb = cross(ad, ab);
					if adb.dot(ao) > 0.0 {
						*cp = *bp;
						*bp = *dp;
						ac = ab;
						ab = ad;
						abc = adb;
						check_tetrahedron![];;
						false
					} else {
						true
					}
				}
			}
		}
		_ => false,
	}
}

struct Tetra<'a>(&'a mut Vec3, &'a mut Vec3, &'a mut Vec3, &'a mut Vec3);

fn check_tetra(te: Tetra, sp: &mut Vec3, w: &mut i32, ao: Vec3, ab: Vec3, ac: Vec3, abc: Vec3) {
	let ab_abc = cross(ab, abc);
	if ab_abc.dot(ao) > 0.0 {
		*te.2 = *te.1;
		*te.1 = *te.0;
		*sp = dcross3(ab, ao);
		*w = 2;
	} else {
		let acp = cross(abc, ac);
		if acp.dot(ao) > 0.0 {
			*te.1 = *te.0;
			*sp = dcross3(ac, ao);
			*w = 2;
		} else {
			*te.3 = *te.2;
			*te.2 = *te.1;
			*te.1 = *te.0;
			*sp = abc;
			*w = 3;
		}
	}
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
	Vec3(a.1 * b.2 - a.2 * b.1,
	     a.2 * b.0 - a.0 * b.2,
	     a.0 * b.1 - a.1 * b.0)
}

fn cross3(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
	cross(cross(a, b), c)
}

fn dcross3(a: Vec3, b: Vec3) -> Vec3 {
	cross3(a, b, a)
}

fn farthest(vertices: &[Vec3], direction: Vec3) -> Vec3 {
	let mut max: Option<f32> = None;
	let mut max_vertex = Vec3::default();
	for vertex in vertices {
		let current = vertex.dot(direction);
		if let Some(value) = max {
			if current > value {
				max = Some(current);
				max_vertex = *vertex;
			}
		} else {
			max = Some(current);
			max_vertex = *vertex;
		}
	}
	max_vertex
}

fn support(vertices_a: &[Vec3], vertices_b: &[Vec3], direction: Vec3) -> Vec3 {
	farthest(vertices_a, direction) - farthest(vertices_b, -direction)
}


#[cfg(test)]
mod tests {

	use std::f32;
	use std::f32::consts::PI;
	use super::{Vec3, bgjk};
	static EPS: f32 = f32::EPSILON;

	macro_rules! pts {
		($($e:expr),*) => {
			[$(
				Vec3($e.0, $e.1, $e.2)
			),*]
		};
	}

	#[test]
	fn square1() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)];
		let shape2 = pts![(-2.0, 0.0, 0.0), (-3.0, 0.0, 0.0), (-2.0, 1.0, 0.0), (-3.0, 1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn exact_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)];
		let shape2 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn line_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
		let shape2 = pts![(0.5, 1.0, 0.0), (0.5, -1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn line_non_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
		let shape2 = pts![(1.5, 1.0, 0.0), (1.5, -1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn small_line_point_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0), (0.01, 0.0, 0.0)];
		let shape2 = pts![(0.005, 0.0, 0.1)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn line_point_non_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
		let shape2 = pts![(0.5, 0.0, 0.1)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn point_overlap() {
		let shape1 = pts![(0.5, 1.0, 0.0)];
		let shape2 = pts![(0.5, 1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn point_no_overlap() {
		let shape1 = pts![(0.5, 1.0, 0.0)];
		let shape2 = pts![(1.0, 1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn empty_no_overlap() {
		// An empty set defaults to a single point in origo in the set
		let shape1: [Vec3; 0] = pts![];
		let shape2 = pts![(1.0, 1.0, 1.0)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn side_by_side_squares() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)];
		let shape2 = pts![(1.0, 0.0, 0.0), (2.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn side_by_side_squares_offset() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)];
		let shape2 =
			pts![(1.0 + EPS, 0.0, 0.0), (2.0, 0.0, 0.0), (1.0 + EPS, 1.0, 0.0), (2.0, 1.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn single_point_square_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 1.0, 0.0)];
		let shape2 = pts![(1.0, 1.0, 0.0), (2.0, 1.0, 0.0), (1.0, 2.0, 0.0), (2.0, 2.0, 0.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn single_point_shape_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0),
		                 (1.0, 0.0, 0.0),
		                 (0.0, 1.0, 0.0),
		                 (1.0, 1.0, 0.0),
		                 (0.0, 0.0, 1.0),
		                 (1.0, 0.0, 1.0),
		                 (0.0, 1.0, 1.0),
		                 (1.0, 1.0, 1.0)];
		let shape2 = pts![(1.0, 1.0, 1.0),
		                 (2.0, 1.0, 1.0),
		                 (1.0, 2.0, 1.0),
		                 (2.0, 2.0, 1.0),
		                 (1.0, 1.0, 2.0),
		                 (2.0, 1.0, 2.0),
		                 (1.0, 2.0, 2.0),
		                 (2.0, 2.0, 2.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn single_point_shape_non_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0),
		                 (1.0, 0.0, 0.0),
		                 (0.0, 1.0, 0.0),
		                 (1.0, 1.0, 0.0),
		                 (0.0, 0.0, 1.0),
		                 (1.0, 0.0, 1.0),
		                 (0.0, 1.0, 1.0),
		                 (1.0, 1.0, 1.0)];
		let shape2 = pts![(1.0, 1.0, 1.0 + EPS),
		                 (2.0, 1.0, 1.0 + EPS),
		                 (1.0, 2.0, 1.0 + EPS),
		                 (2.0, 2.0, 1.0 + EPS),
		                 (1.0, 1.0, 2.0),
		                 (2.0, 1.0, 2.0),
		                 (1.0, 2.0, 2.0),
		                 (2.0, 2.0, 2.0)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn single_line_shape_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0),
		                 (1.0, 0.0, 0.0),
		                 (0.0, 1.0, 0.0),
		                 (1.0, 1.0, 0.0),
		                 (0.0, 0.0, 1.0),
		                 (1.0, 0.0, 1.0),
		                 (0.0, 1.0, 1.0),
		                 (1.0, 1.0, 1.0)];
		let shape2 = pts![(1.0, 1.0, 0.0),
		                 (2.0, 1.0, 0.0),
		                 (1.0, 2.0, 0.0),
		                 (2.0, 2.0, 0.0),
		                 (1.0, 1.0, 1.0),
		                 (2.0, 1.0, 1.0),
		                 (1.0, 2.0, 1.0),
		                 (2.0, 2.0, 1.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn shape_projective_non_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0),
		                 (1.0, 0.0, 0.0),
		                 (0.0, 1.0, 0.0),
		                 (1.0, 1.0, 0.0),
		                 (1.0, 0.0, 1.0),
		                 (2.0, 0.0, 1.0),
		                 (1.0, 1.0, 1.0),
		                 (2.0, 1.0, 1.0)];
		let shape2 = pts![(1.1, 1.0, 0.0),
		                 (2.1, 1.0, 0.0),
		                 (1.1, 2.0, 0.0),
		                 (2.1, 2.0, 0.0),
		                 (2.1, 1.0, 1.0),
		                 (3.1, 1.0, 1.0),
		                 (2.1, 2.0, 1.0),
		                 (3.1, 2.0, 1.0)];
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn shape_projective_overlap() {
		let shape1 = pts![(0.0, 0.0, 0.0),
		                 (1.0, 0.0, 0.0),
		                 (0.0, 1.0, 0.0),
		                 (1.0, 1.0, 0.0),
		                 (1.0, 0.0, 1.0),
		                 (2.0, 0.0, 1.0),
		                 (1.0, 1.0, 1.0),
		                 (2.0, 1.0, 1.0)];
		let shape2 = pts![(1.1, 1.0, 0.0),
		                 (2.1, 1.0, 0.0),
		                 (1.1, 2.0, 0.0),
		                 (2.1, 2.0, 0.0),
		                 (2.0, 1.0, 1.0),
		                 (3.1, 1.0, 1.0),
		                 (2.0, 2.0, 1.0),
		                 (3.1, 2.0, 1.0)];
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn shape_non_overlap() {
		let (mut shape1, mut shape2) = (vec![], vec![]);
		let units = 100;
		shape1.reserve(units);
		shape2.reserve(units);
		for i in 0..units {
			let radian = i as f32 / units as f32 * 2.0 * PI;
			shape1.push(Vec3(radian.cos(), radian.sin(), 0.0));
			shape2.push(Vec3(radian.cos(), radian.sin(), EPS));
		}
		assert_eq![bgjk(&shape1, &shape2), false];
	}

	#[test]
	fn shape_overlap() {
		let (mut shape1, mut shape2) = (vec![], vec![]);
		let units = 100;
		shape1.reserve(units);
		shape2.reserve(units);
		for i in 0..units {
			let radian = i as f32 / units as f32 * 2.0 * PI;
			shape1.push(Vec3(radian.cos(), radian.sin(), 0.0));
			shape2.push(Vec3(radian.cos(), radian.sin(), 0.0));
		}
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn shape_section() {
		let (mut shape1, mut shape2) = (vec![], vec![]);
		let units = 100;
		shape1.reserve(units);
		shape2.reserve(units);
		for i in 0..units {
			let radian = i as f32 / units as f32 * 2.0 * PI;
			shape1.push(Vec3(radian.cos(), radian.sin(), 0.0));
			shape2.push(Vec3(radian.cos() + 0.5, radian.sin(), 0.0));
		}
		assert_eq![bgjk(&shape1, &shape2), true];
	}

	#[test]
	fn shape_away() {
		let (mut shape1, mut shape2) = (vec![], vec![]);
		let units = 100;
		shape1.reserve(units);
		shape2.reserve(units);
		for i in 0..units {
			let radian = i as f32 / units as f32 * 2.0 * PI;
			shape1.push(Vec3(radian.cos(), radian.sin(), 0.0));
			shape2.push(Vec3(radian.cos() + 2.0 + 2.0 * EPS, radian.sin(), 0.0));
		}
		assert_eq![bgjk(&shape1, &shape2), false];
	}

}
