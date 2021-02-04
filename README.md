# Bounding Box Generation for Bevy

Unofficial plugin for generating bounding boxes.

| Status | Bounding Volume |
|:-:|----------------------------|
| WIP 🚧 | Bounding Sphere            |
| WIP 🚧 | Axis Aligned Bounding Box  |
| WIP 🚧 | Oriented Bounding Box      |
| Not Started ❌ | Convex Hull                |

## Notes

Need to redesign.

* unique type per bounding box type. Dedicated systems for each bounding system type - can optimize them differently.
* Bounding boxes should be children of mesh. All but AABB don't need to recompute on 

Bounding volume recalculation needed when the following change:

BS
1. Mesh change
2. Scale

AABB
1. Mesh
2. Scale
3. Rotation

OBB/DOP/CV
1. Mesh

For bound meshes for BS and ABB, compute the size of the mesh with these properties applied, then apply inverse of the properties to the mesh, so the bounding box is in the correct place when the globaltransform is applied to the mesh.

The properties not listed would then be applied to the bounding box calculation lazily when it is requested.
