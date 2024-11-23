use crate::points3d::*;
use fxhash::{FxHashMap, FxHashSet};



macro_rules! impl_index {
  ($el: ty, $i: ty) => {
    impl std::ops::Index<$i> for Vec<$el> {
        type Output = $el;
        fn index(&self, index: $i) -> &Self::Output {
            self.index(index.0)
        }
    }

    impl std::ops::IndexMut<$i> for Vec<$el> {
        fn index_mut(&mut self, index: $i) -> &mut Self::Output {
            self.index_mut(index.0)
        }
    }
  }
}

struct PointID(usize);
impl_index!(Point, PointID);
struct LineID(usize);
impl_index!(Line, LineID);
struct PlaneID(usize);
impl_index!(Plane, PlaneID);
struct FaceID(usize);
impl_index!(Face, FaceID);
struct RoomID(usize);
impl_index!(Room, RoomID);
struct PartitionNodeID(usize);
impl_index!(PartitionNode, PartitionNodeID);

pub struct Plane {
    points: FxHashSet<PointID>, // Sum of edges's points
    lines: FxHashSet<LineID>,
    intersections_with_planes: FxHashMap<PlaneID, LineID>,
    intersections_with_edges: FxHashMap<LineID, PointID>,
}

pub struct Line {
    points: FxHashSet<PointID>,
}

pub struct Vertex {
    point: PointID,
    adj_face: FaceID,
    line: LineID,
}

pub struct Face {
    plane: PlaneID,
    vertices: Vec<Vertex>,
    next_side_room: RoomID,
    next_side_face: FaceID, // inside next room
}

pub struct Room {
    faces: Vec<Face>,
}

enum PartitionVariant {
    PartitionNode(PartitionNodeID),
    Room(RoomID),
}

pub struct PartitionNode {
    cutting_plane: PlaneID,
    positive: PartitionVariant,
    negative: PartitionVariant,
}

pub struct Scene {
    points: Vec<Point>,
    lines: Vec<Line>,
    planes: Vec<Plane>,
    rooms: Vec<Room>,
    nodes: Vec<PartitionNode>,
}


impl Scene {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            lines: Vec::new(),
            planes: Vec::new(),
            rooms: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn generate_grid(size: f32, count: usize) -> Self {
        Self::new()
    }

    pub fn cut_by_face(&mut self, face: &Face) {
    }
}
