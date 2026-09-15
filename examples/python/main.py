import vectory


fc = vectory.FeatureCollection(
    datum=(52.0, 5.0, 0.0),
    heading=(0.0, 0.0, 1.5),
)
fc.set_global_property("field_id", "F42")
fc.add_point_feature((1.0, 2.0, 3.0), {"name": "point_a"})
fc.add_segment_feature((0.0, 0.0, 0.0), (5.0, 5.0, 0.0), {"name": "segment_a"})
print("feature_count:", fc.feature_count())
print("json:", fc.to_json("ENU"))

vec = vectory.Vector(
    field_boundary=[
        (0.0, 0.0, 0.0),
        (10.0, 0.0, 0.0),
        (10.0, 10.0, 0.0),
        (0.0, 10.0, 0.0),
    ],
    datum=(52.0, 5.0, 0.0),
    heading=(0.0, 0.0, 0.0),
    crs="ENU",
)
vec.set_field_property("name", "Python Demo Field")
vec.add_point((2.0, 3.0, 0.0), "marker", {"label": "A"})
vec.add_polygon(
    [(1.0, 1.0, 0.0), (3.0, 1.0, 0.0), (3.0, 3.0, 0.0), (1.0, 3.0, 0.0)],
    "zone",
    {"kind": "test"},
)
print("vector_count:", vec.element_count())
print("vector_elements:", vec.elements())
print("global_property:", vec.get_global_property("missing", "unset"))
