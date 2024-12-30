mod range_0d;
mod range_1d;
mod range_2d;
mod range_3d;
mod range_iter;

pub use range_1d::*;
pub use range_2d::*;
pub use range_3d::*;

use bvh::aabb::Aabb;

/// Trait for range.
pub trait Range {
    /// Gets points in the range.
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)>;
    /// Gets axis-aligned bounding box of the range.
    fn aabb(&self) -> Aabb<f32, 3>;
}
