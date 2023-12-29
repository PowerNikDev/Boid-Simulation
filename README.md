# Boid Simulation using Quadtrees
#### Video Demo:  https://www.youtube.com/watch?v=jGHyrR0L-AM)https://www.youtube.com/watch?v=jGHyrR0L-AM
#### Description: This is my CS50x final project. I used the rust programming language to implement the artifical life program "boids" using the quad tree datatstructure. For the graphics I used the bevy game engine. This is not the most performant simulation possible. There are still some things to be improved. The simulation doesn't use multithreading and only runs on one CPU core. 

#### Features:
- The simulation includes points of attraction, to which all boids are attracted. You can place them with the left mouse button and remove them with the right mouse button.

#### Dependencies:
- Bevy version 0.11
- Rustup version 1.75

#### Design Choices:
- All the settings for the simulation, i.e. the amount of boids, the seperation factor, the cohesion factor, etc., are stored in a bevy resource, so that it can be accessed in every bevy system
- A boid consists out of three components: a MaterialMesh2DBundle, a self-made Movement Component, storing the position and velocity of the boid, and a Boid Component storing a unique ID for every boid
- The simulation has four main systems:
  - The simulation system: The system calculates the new velocity of every boid and applies it after the computation
  - The apply-velocity system: The system applies the velocity to the position and updates the Quadtree
  - The draw-boids system: The system updates the position of the boids on screen
  - The check-input system: The system handles the creation and deletion of points of attraction
- The boids turn away from the edge of the screen as soon as they are only 100px away

#### Qustionable Design Choices:
- The quadtree doesn't save e.g. a pointer to a boid, but two Vec2. Those save the position and the velocity of a boid. I did this, because it is not necessary to have actual mutable access to the other boid, but only the information of position and velocity to compute the change in velocity of the current boid. Also, it would be complicated to implement considering Rust's lifetime rules and Bevy's way of storing entities.

#### Future Ideas:
- Precache the Quadtree to a certain depth, so the programm doesn't have to allocate memory every time a boid moves in a different Quadtree.
- Add sliders to dynamically change the settings of the simulation

#### Bugs:
- When resizing the window, the 0, 0 point isn't at the center of the window anymore, thus the boids are won't avoid the window edges correctly anymore
- This probably a bug in the Bevy Game Engine and not in my code, so maybe it will be fixed in a future update to the engine
