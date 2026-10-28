use super::*;
use crate::frame::Frame;
use crate::vector::Quaternion;

const CELL: u32 = 0x7F7F_0001;

fn at(x: f32, y: f32, z: f32) -> Position {
    Position::new(CELL, Frame::new(Vector::new(x, y, z), Quaternion::IDENTITY))
}

fn volume(shape: impl FnOnce(&mut DetectionVolume)) -> DetectionVolume {
    let mut v = DetectionVolume::new();
    v.position = at(100.0, 100.0, 50.0);
    shape(&mut v);
    v
}

fn unit_sphere(x: f32, y: f32, z: f32) -> Sphere {
    Sphere {
        center: Vector::new(x, y, z),
        radius: 1.0,
    }
}

#[test]
fn a_box_reaches_each_way_by_its_sides() {
    let v = volume(|v| v.create_bbox(4.0, 3.0, 2.0, 1.0, 6.0, 5.0));
    let Some(DetectionShape::BoundBox(b)) = v.shape() else {
        panic!("box");
    };
    assert_eq!(
        (b.min, b.max),
        (Vector::new(-1.0, -3.0, -5.0), Vector::new(2.0, 4.0, 6.0))
    );
}

#[test]
fn a_sphere_volume_meets_a_sphere_that_touches_it() {
    let v = volume(|v| v.create_sphere(2.0, Vector::ZERO));
    let here = at(100.0, 100.0, 50.0);
    assert!(v.is_within_sphere(
        &unit_sphere(2.5, 0.0, 0.0),
        &here,
        Vector::new(1.0, 1.0, 1.0)
    ));
    assert!(!v.is_within_sphere(
        &unit_sphere(3.5, 0.0, 0.0),
        &here,
        Vector::new(1.0, 1.0, 1.0)
    ));
}

#[test]
fn a_sphere_is_moved_into_the_volumes_frame() {
    let v = volume(|v| v.create_sphere(2.0, Vector::ZERO));
    // The object stands 10 m east of the volume: its centre is 10 m away in the volume's frame.
    let there = at(110.0, 100.0, 50.0);
    assert!(!v.is_within_sphere(
        &unit_sphere(0.0, 0.0, 0.0),
        &there,
        Vector::new(1.0, 1.0, 1.0)
    ));
    assert!(v.is_within_sphere(
        &unit_sphere(-9.0, 0.0, 0.0),
        &there,
        Vector::new(1.0, 1.0, 1.0)
    ));
}

#[test]
fn a_non_uniform_scale_is_ignored() {
    let v = volume(|v| v.create_sphere(2.0, Vector::ZERO));
    let here = at(100.0, 100.0, 50.0);
    // Scaled by 0.5 the centre at 6 would be 3 away; the uneven scale leaves it at 6.
    assert!(!v.is_within_sphere(
        &unit_sphere(6.0, 0.0, 0.0),
        &here,
        Vector::new(0.5, 1.0, 0.5)
    ));
    assert!(v.is_within_sphere(
        &unit_sphere(3.0, 0.0, 0.0),
        &here,
        Vector::new(1.0, 1.0, 1.0)
    ));
}

#[test]
fn a_box_volume_meets_spheres_cylinders_and_boxes() {
    let v = volume(|v| v.create_bbox(5.0, 5.0, 5.0, 5.0, 5.0, 5.0));
    let here = at(100.0, 100.0, 50.0);
    assert!(v.is_within_sphere(
        &unit_sphere(5.5, 0.0, 0.0),
        &here,
        Vector::new(1.0, 1.0, 1.0)
    ));
    assert!(!v.is_within_sphere(
        &unit_sphere(6.5, 0.0, 0.0),
        &here,
        Vector::new(1.0, 1.0, 1.0)
    ));
    let cyl = CylSphere {
        low: Vector::new(5.5, 0.0, 0.0),
        height: 2.0,
        radius: 1.0,
    };
    assert!(v.is_within_cylinder(&cyl, &here));
    let high = CylSphere {
        low: Vector::new(0.0, 0.0, 9.0),
        ..cyl
    };
    assert!(!v.is_within_cylinder(&high, &here));
    let near = BBox {
        min: Vector::new(4.0, 4.0, 4.0),
        max: Vector::new(7.0, 7.0, 7.0),
    };
    let far = BBox {
        min: Vector::new(6.0, 6.0, 6.0),
        max: Vector::new(7.0, 7.0, 7.0),
    };
    assert!(v.is_within_box(&near, &here));
    assert!(!v.is_within_box(&far, &here));
}

#[test]
fn a_cylinder_volume_meets_spheres_cylinders_and_boxes() {
    let v = volume(|v| v.create_cylinder(2.0, Vector::ZERO, 4.0));
    let here = at(100.0, 100.0, 50.0);
    // Above the top the sphere reaches down by its radius.
    assert!(v.is_within_sphere(
        &unit_sphere(0.0, 0.0, 4.5),
        &here,
        Vector::new(1.0, 1.0, 1.0)
    ));
    assert!(!v.is_within_sphere(
        &unit_sphere(0.0, 0.0, 8.0),
        &here,
        Vector::new(1.0, 1.0, 1.0)
    ));
    let beside = CylSphere {
        low: Vector::new(3.5, 0.0, 1.0),
        height: 1.0,
        radius: 1.5,
    };
    assert!(v.is_within_cylinder(&beside, &here));
    let box_in = BBox {
        min: Vector::new(1.0, -1.0, 1.0),
        max: Vector::new(3.0, 1.0, 2.0),
    };
    assert!(v.is_within_box(&box_in, &here));
}

#[test]
fn an_unplaced_volume_has_no_global_box() {
    let mut v = DetectionVolume::new();
    v.create_sphere(2.0, Vector::ZERO);
    assert_eq!(v.global_box(), None);
    v.position = at(100.0, 100.0, 50.0);
    let b = v.global_box().expect("placed");
    assert_eq!(
        (b.min, b.max),
        (
            Vector::new(98.0, 98.0, 48.0),
            Vector::new(102.0, 102.0, 52.0)
        )
    );
}

#[test]
fn a_cylinders_global_box_is_its_end_points_grown_by_the_radius() {
    let v = volume(|v| v.create_cylinder(1.0, Vector::new(1.0, 0.0, 0.0), 3.0));
    let b = v.global_box().expect("placed");
    assert_eq!(
        (b.min, b.max),
        (
            Vector::new(100.0, 99.0, 50.0),
            Vector::new(102.0, 101.0, 53.0)
        )
    );
}

#[test]
fn a_boxs_global_box_is_the_box_moved_to_the_volume() {
    let v = volume(|v| v.create_bbox(1.0, 1.0, 1.0, 1.0, 2.0, 0.0));
    let b = v.global_box().expect("placed");
    assert_eq!(
        (b.min, b.max),
        (
            Vector::new(99.0, 99.0, 50.0),
            Vector::new(101.0, 101.0, 52.0)
        )
    );
}

#[test]
fn box_within_includes_the_faces() {
    let b = BBox {
        min: Vector::ZERO,
        max: Vector::new(1.0, 1.0, 1.0),
    };
    assert!(box_within(&b, Vector::new(1.0, 0.0, 0.5)));
    assert!(!box_within(&b, Vector::new(1.1, 0.0, 0.5)));
}
