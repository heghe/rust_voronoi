# Generating Voronoi diagram

### Introduction

Voronoi diagram is a partition of a multi dimension space into regions based on distance to points in a specific subset of the space. The set of points (also called seeds, sites, or generators) is specified beforehand, and for each point there is a coresponding space region with the property that all the points in that region is closer to it's seed than to any other. This regions are called Voronoi cells.

Most general space used for Voronoi diagrams it's the 2D space, having multiple applications in various domains.

#### Generating Voronoi diagrams in 2D space _(possible also 3D space)_

##### Input data (_files_)
 - __X, Y__: dimension of the 2D space
 - __n__: unsigned integer representing the number of seeds
 - on the following __n__ lines there are pairs of __x, y__ representing the position of the seeds in the _application space dimension_
 
The __application space dimension__ it's the space with the constrains that __x__ is an integer in [0, __X__] and __y__ is an integer in [0,__Y__]

##### Output
An image representing the Voronoi diagram, using as distancefunction the euclidian distance.
*also for debug purpose(and maybe easy comparison of results between different implementations) there might be an aditional output file.

#### Sequence implementation algorithm
The classic flood fill algorithm where at each step, for each seed I'll go one more tile in the 8 direction, until all the tiles are filled (meaning that I found the closest seed for the tile).

![ ](extra/steps/steps.png  "Sequence example")

