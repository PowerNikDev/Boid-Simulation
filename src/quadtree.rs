use bevy::prelude::*;


// Implementation of a simple data structure to store a rectangle

#[derive(Debug)]
pub struct Rectangle
{
    pub position: Vec2,

    // For consitency the size of the rectangle is also stored in an Vec2
    pub size: Vec2,
}

impl Rectangle
{

}


// Implementation of the Quad Tree data structure
#[derive(Debug)]
pub struct QuadTree
{
    boundary: Rectangle,
    capacity: usize,
    points: Vec<Vec2>,
    pub quads: Vec<QuadTree>,
    divided: bool,
}

impl QuadTree
{
    pub fn new(boundary: Rectangle, capacity: usize) -> QuadTree
    {
        QuadTree{boundary: boundary, capacity: capacity, points: Vec::with_capacity(capacity), quads: Vec::with_capacity(4), divided: false}
    }

    // Inserts a new point or in the children
    pub fn insert(&mut self, point: Vec2)
    {
        // If the current quad still has enough capacitz, it will store the new point, otherwise it will subdivide
        // The current quad will still keep their existing points and not give them to its children
        if self.points.len() < self.capacity
        {
            self.points.push(point);
        }
        else {
            if !self.divided
            {
                println!("Subdivide");
                self.subdivide();
            }

            // This may seem like a mistake, why inserting the point in every of the cildren?
            // It works, because every child checks if the point is within its boundarz
            self.quads[0].insert(point);
            self.quads[1].insert(point);
            self.quads[2].insert(point);
            self.quads[3].insert(point);
        }
    
    }

    fn subdivide(&mut self)
    {
        let northwest = Rectangle {
            position: Vec2 { x: self.boundary.position.x + (self.boundary.size.x / 4.0), y: self.boundary.position.y + (self.boundary.size.y / 4.0) },
            size: self.boundary.size / 2.0
        };

        let northeast = Rectangle {
            position: Vec2 { x: self.boundary.position.x - (self.boundary.size.x / 4.0), y: self.boundary.position.y + (self.boundary.size.y / 4.0) },
            size: self.boundary.size / 2.0
        };

        let southwest = Rectangle {
            position: Vec2 { x: self.boundary.position.x + (self.boundary.size.x / 4.0), y: self.boundary.position.y - (self.boundary.size.y / 4.0) },
            size: self.boundary.size / 2.0
        };

        let southeast = Rectangle {
            position: Vec2 { x: self.boundary.position.x - (self.boundary.size.x / 4.0), y: self.boundary.position.y - (self.boundary.size.y / 4.0) },
            size: self.boundary.size / 2.0
        };
        
        self.quads.push(QuadTree::new(northwest, self.capacity));
        self.quads.push(QuadTree::new(northeast, self.capacity));
        self.quads.push(QuadTree::new(southwest, self.capacity));
        self.quads.push(QuadTree::new(southeast, self.capacity));
        self.divided = true;
    }

} 