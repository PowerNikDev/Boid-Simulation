use bevy::{prelude::*, app::Plugin};


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
    // Check if an point is within the rectangle
    fn contains(&self, point: &Vec2) -> bool
    {
        return 
            point.x >= self.position.x - self.size.x &&
            point.x < self.position.x + self.size.x &&
            point.y >= self.position.y - self.size.y &&
            point.y < self.position.y + self.size.y
    }

    fn intersects(&self, region: &Rectangle) -> bool
    {
        return
            !(region.position.x - region.size.x > self.position.x + self.size.x ||
            region.position.x + region.size.x < self.position.x - self.size.x ||
            region.position.y - region.size.y > self.position.y + self.size.y ||
            region.position.y + region.size.y < self.position.y - self.size.y) 
    }
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

            if self.quads[0].boundary.contains(&point)
            {
                self.quads[0].insert(point);
            }
            else if self.quads[1].boundary.contains(&point) 
            {
                self.quads[1].insert(point);
            }
            else if self.quads[2].boundary.contains(&point)
            {
                self.quads[2].insert(point);
            }
            else if self.quads[3].boundary.contains(&point)
            {
                self.quads[3].insert(point);
            }
        }
    
    }

    pub fn query(&self, region: &Rectangle) -> Vec<&Vec2>
    {
        if !self.boundary.intersects(&region)
        {
            return vec![]
        }

        let mut found: Vec<&Vec2> = vec![];

        for point in &self.points
        {
            if region.contains(&point)
            {
                found.push(point);
            }
        }

        for quad in &self.quads
        {
            found.append(&mut quad.query(region));
        }

        return found
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