#include <stdio.h>

#include "../../include/vectory.h"

int main(void) {
  const char* json =
      "{"
      "\"type\":\"FeatureCollection\","
      "\"properties\":{\"crs\":\"ENU\",\"datum\":[5.0,52.0,0.0],\"heading\":0.0},"
      "\"features\":[{"
      "\"type\":\"Feature\","
      "\"geometry\":{\"type\":\"Point\",\"coordinates\":[1.0,2.0,3.0]},"
      "\"properties\":{\"name\":\"demo_point\"}"
      "}]"
      "}";

  VectoryFeatureCollection* fc = vectory_feature_collection_from_json(json);
  if (fc == NULL) {
    fprintf(stderr, "from_json failed: %s\n", vectory_last_error_message());
    return 1;
  }

  printf("feature_count=%zu\n", vectory_feature_collection_feature_count(fc));

  VectoryPoint3 point;
  if (!vectory_feature_collection_feature_point(fc, 0, &point)) {
    fprintf(stderr, "feature_point failed: %s\n", vectory_last_error_message());
    vectory_feature_collection_free(fc);
    return 1;
  }
  printf("point=(%.3f, %.3f, %.3f)\n", point.x, point.y, point.z);

  char* encoded = vectory_feature_collection_to_json(fc, VECTORY_CRS_ENU);
  if (encoded == NULL) {
    fprintf(stderr, "to_json failed: %s\n", vectory_last_error_message());
    vectory_feature_collection_free(fc);
    return 1;
  }
  printf("json=%s\n", encoded);
  vectory_string_free(encoded);
  vectory_feature_collection_free(fc);

  VectoryPoint3 boundary_points[4] = {
      {0.0, 0.0, 0.0},
      {10.0, 0.0, 0.0},
      {10.0, 10.0, 0.0},
      {0.0, 10.0, 0.0},
  };

  VectoryVector* vector = vectory_vector_new(
      (VectoryPointArrayView){boundary_points, 4},
      (VectoryGeo3){52.0, 5.0, 0.0},
      (VectoryEuler){0.0, 0.0, 0.0},
      VECTORY_CRS_ENU);
  if (vector == NULL) {
    fprintf(stderr, "vector_new failed: %s\n", vectory_last_error_message());
    return 1;
  }

  if (!vectory_vector_add_point(vector, (VectoryPoint3){5.0, 5.0, 0.0}, "marker")) {
    fprintf(stderr, "vector_add_point failed: %s\n", vectory_last_error_message());
    vectory_vector_free(vector);
    return 1;
  }

  printf("vector_elements=%zu\n", vectory_vector_element_count(vector));
  vectory_vector_free(vector);
  return 0;
}
