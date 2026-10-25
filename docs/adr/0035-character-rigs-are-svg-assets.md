# Character rigs are explicit SVG-backed assets

Static illustrations remain `scene.svg(...)`; articulated figures use a separate `scene.character(...)` API so ordinary SVG imports keep their current whole-art meaning. A Character's SVG defines named parts, nested parent-child joints, viewBox-space pivots, and named expression variants. Authors set its pose in each Scene, and Codimate applies the hierarchy and its existing Scene-to-Scene tweening. Ship the Codimate mascot as `cm.characters.codimate` alongside support for user-authored rigged SVGs. Joint handles rotate around their authored pivots; placement and scale belong to the Character root.

Character color variants are also authored in the SVG. A joint may provide
`data-dark-fill="#rrggbb"`; its normal SVG fill remains the light palette, and
`character.theme("dark")` selects the alternate paint for that Scene. Themes
belong to the Character pose because the Authoring Surface has no global scene
theme. Color changes follow the same Scene-to-Scene interpolation as the pose.
