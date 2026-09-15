import vectory


fc = vectory.FeatureCollection.from_file(
    "/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson"
)
print("datum:", fc.datum())
print("heading:", fc.heading())
print("feature_count:", fc.feature_count())
print("first_feature:", fc.feature(0))

vec = vectory.Vector.from_file("/home/bresilla/data/code/robolibs/vectkit/misc/field4.geojson")
print("vector_elements:", vec.element_count())
print("field_properties:", vec.field_properties())
