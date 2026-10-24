use ac2_core::Sidedness;
use glam::Vec3;

const LEGACY_EPSILON: f32 = 0.0002;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub d: f32,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            normal: Vec3::Y,
            d: 0.0,
        }
    }
}

impl Plane {
    pub const fn new(normal: Vec3, d: f32) -> Self {
        Self { normal, d }
    }

    pub fn from_normal_and_point(normal: Vec3, point: Vec3) -> Self {
        Self {
            normal,
            d: -normal.dot(point),
        }
    }

    pub fn normalized(self) -> Self {
        let length = self.normal.length();
        if length == 0.0 {
            return self;
        }

        Self {
            normal: self.normal / length,
            d: self.d / length,
        }
    }

    pub fn signed_distance_to_point(self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.d
    }

    pub fn approximately_eq(self, other: Self, epsilon: f32) -> bool {
        self.normal.abs_diff_eq(other.normal, epsilon) && (self.d - other.d).abs() <= epsilon
    }

    pub fn which_side(self, point: Vec3, epsilon: f32) -> Sidedness {
        side_from_distance(self.signed_distance_to_point(point), epsilon)
    }

    pub fn which_side_biased(self, point: Vec3, bias: f32, epsilon: f32) -> Sidedness {
        side_from_distance(self.signed_distance_to_point(point) + bias, epsilon)
    }

    pub fn snap_point_to_plane_height(self, mut point: Vec3) -> Option<Vec3> {
        if self.normal.z.abs() <= LEGACY_EPSILON {
            return None;
        }

        point.z = -((point.y * self.normal.y + point.x * self.normal.x + self.d) / self.normal.z);
        Some(point)
    }

    pub fn set_height(self, point: &mut Vec3) -> bool {
        if let Some(adjusted) = self.snap_point_to_plane_height(*point) {
            point.z = adjusted.z;
            true
        } else {
            false
        }
    }

    pub fn ray_intersection_time(self, ray: Ray, epsilon: f32) -> Option<f32> {
        let denominator = self.normal.dot(ray.direction);
        if denominator.abs() < epsilon {
            return None;
        }

        let time = -self.signed_distance_to_point(ray.origin) / denominator;
        (time >= 0.0).then_some(time)
    }

    pub fn intersect_bbox(self, bbox: BBox, epsilon: f32) -> Sidedness {
        let mut corners = bbox.corners().into_iter();
        let first = self.which_side(corners.next().expect("bbox has corners"), epsilon);
        if first == Sidedness::IN_PLANE {
            return Sidedness::CROSSING;
        }

        for corner in corners {
            let side = self.which_side(corner, epsilon);
            if side == Sidedness::IN_PLANE || side != first {
                return Sidedness::CROSSING;
            }
        }

        first
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sphere {
    pub origin: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub const fn new(origin: Vec3, radius: f32) -> Self {
        Self { origin, radius }
    }

    pub fn contains_point(self, point: Vec3) -> bool {
        self.origin.distance_squared(point) <= self.radius * self.radius
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        }
    }
}

impl Ray {
    pub const fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn point_at(self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BBox {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn for_adjustment() -> Self {
        Self {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
    }

    pub fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(self) -> Vec3 {
        self.max - self.min
    }

    pub fn contains_point(self, point: Vec3) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }

    pub fn expand_to_include_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn expand_to_include_bbox(&mut self, other: Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn union(self, other: Self) -> Self {
        let mut union = self;
        union.expand_to_include_bbox(other);
        union
    }

    fn corners(self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
        ]
    }
}

fn side_from_distance(distance: f32, epsilon: f32) -> Sidedness {
    if distance > epsilon {
        Sidedness::POSITIVE
    } else if distance < -epsilon {
        Sidedness::NEGATIVE
    } else {
        Sidedness::IN_PLANE
    }
}

#[cfg(test)]
mod tests {
    use ac2_core::Sidedness;
    use glam::Vec3;

    use super::{BBox, Plane, Ray, Sphere};

    #[test]
    fn normalized_plane_preserves_distance() {
        let plane = Plane::new(Vec3::new(0.0, 2.0, 0.0), -4.0).normalized();
        assert!((plane.normal.length() - 1.0).abs() < 0.0001);
        assert!((plane.signed_distance_to_point(Vec3::new(0.0, 2.0, 0.0))).abs() < 0.0001);
    }

    #[test]
    fn plane_side_queries_match_sidedness_constants() {
        let plane = Plane::new(Vec3::Z, 0.0);

        assert_eq!(
            plane.which_side(Vec3::new(0.0, 0.0, 2.0), 0.0001),
            Sidedness::POSITIVE
        );
        assert_eq!(
            plane.which_side(Vec3::new(0.0, 0.0, -2.0), 0.0001),
            Sidedness::NEGATIVE
        );
        assert_eq!(plane.which_side(Vec3::ZERO, 0.0001), Sidedness::IN_PLANE);
        assert_eq!(
            plane.which_side_biased(Vec3::new(0.0, 0.0, -0.25), 0.5, 0.0001),
            Sidedness::POSITIVE
        );
    }

    #[test]
    fn plane_intersect_bbox_reports_crossing_when_span_crosses_plane() {
        let plane = Plane::new(Vec3::Z, 0.0);
        let crossing = BBox::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let above = BBox::new(Vec3::new(-1.0, -1.0, 1.0), Vec3::new(1.0, 1.0, 2.0));

        assert_eq!(plane.intersect_bbox(crossing, 0.0001), Sidedness::CROSSING);
        assert_eq!(plane.intersect_bbox(above, 0.0001), Sidedness::POSITIVE);
    }

    #[test]
    fn plane_can_project_and_intersect_rays() {
        let plane = Plane::from_normal_and_point(Vec3::Z, Vec3::new(0.0, 0.0, 5.0));
        let mut point = Vec3::new(1.0, 2.0, 99.0);
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);

        assert!(plane.set_height(&mut point));
        assert_eq!(point.z, 5.0);
        assert_eq!(plane.ray_intersection_time(ray, 0.0001), Some(5.0));
    }

    #[test]
    fn bbox_adjustment_accumulator_expands_by_point_and_bbox() {
        let mut bbox = BBox::for_adjustment();
        bbox.expand_to_include_point(Vec3::new(3.0, -1.0, 5.0));
        bbox.expand_to_include_point(Vec3::new(-2.0, 4.0, 1.0));
        bbox.expand_to_include_bbox(BBox::new(
            Vec3::new(-4.0, 0.0, -3.0),
            Vec3::new(1.0, 8.0, 2.0),
        ));

        assert_eq!(bbox.min, Vec3::new(-4.0, -1.0, -3.0));
        assert_eq!(bbox.max, Vec3::new(3.0, 8.0, 5.0));
    }

    #[test]
    fn sphere_contains_points_inside_radius() {
        let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 2.0);
        assert!(sphere.contains_point(Vec3::new(2.0, 2.0, 3.0)));
        assert!(!sphere.contains_point(Vec3::new(4.1, 2.0, 3.0)));
    }

    #[test]
    fn ray_point_at_moves_along_direction() {
        let ray = Ray::new(Vec3::ONE, Vec3::new(0.0, 0.0, 2.0));
        assert_eq!(ray.point_at(3.0), Vec3::new(1.0, 1.0, 7.0));
    }

    #[test]
    fn bbox_contains_and_unions_points() {
        let left = BBox::new(Vec3::ZERO, Vec3::splat(2.0));
        let right = BBox::new(Vec3::splat(1.0), Vec3::splat(4.0));

        assert!(left.contains_point(Vec3::new(1.0, 0.5, 2.0)));
        assert!(!left.contains_point(Vec3::new(3.0, 0.0, 0.0)));

        let union = left.union(right);
        assert_eq!(union.min, Vec3::ZERO);
        assert_eq!(union.max, Vec3::splat(4.0));
    }
}
