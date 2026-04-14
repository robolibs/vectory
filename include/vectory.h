#ifndef VECTORY_H
#define VECTORY_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct VectoryFeatureCollectionHandle VectoryFeatureCollectionHandle;
typedef struct VectoryVectorHandle VectoryVectorHandle;

typedef struct {
  double latitude;
  double longitude;
  double altitude;
} VectoryGeo3;

typedef struct {
  double roll;
  double pitch;
  double yaw;
} VectoryEuler;

typedef struct {
  double x;
  double y;
  double z;
} VectoryPoint3;

typedef struct {
  VectoryPoint3 start;
  VectoryPoint3 end;
} VectorySegment3;

typedef struct {
  const VectoryPoint3* ptr;
  size_t len;
} VectoryPointArrayView;

typedef enum {
  VECTORY_CRS_WGS = 0,
  VECTORY_CRS_ENU = 1,
} VectoryCrs;

typedef enum {
  VECTORY_GEOMETRY_NONE = 0,
  VECTORY_GEOMETRY_POINT = 1,
  VECTORY_GEOMETRY_SEGMENT = 2,
  VECTORY_GEOMETRY_PATH = 3,
  VECTORY_GEOMETRY_POLYGON = 4,
} VectoryGeometryKind;

const char* vectory_last_error_message(void);
void vectory_string_free(char* ptr);

VectoryFeatureCollectionHandle* vectory_feature_collection_new(VectoryGeo3 datum, VectoryEuler heading);
void vectory_feature_collection_free(VectoryFeatureCollectionHandle* handle);
VectoryFeatureCollectionHandle* vectory_feature_collection_read(const char* path);
VectoryFeatureCollectionHandle* vectory_feature_collection_from_json(const char* text);
bool vectory_feature_collection_write(
    const VectoryFeatureCollectionHandle* handle,
    const char* path,
    VectoryCrs crs);
char* vectory_feature_collection_to_json(
    const VectoryFeatureCollectionHandle* handle,
    VectoryCrs crs);
size_t vectory_feature_collection_feature_count(const VectoryFeatureCollectionHandle* handle);
bool vectory_feature_collection_get_datum(
    const VectoryFeatureCollectionHandle* handle,
    VectoryGeo3* out);
bool vectory_feature_collection_set_datum(
    VectoryFeatureCollectionHandle* handle,
    VectoryGeo3 datum);
bool vectory_feature_collection_get_heading(
    const VectoryFeatureCollectionHandle* handle,
    VectoryEuler* out);
bool vectory_feature_collection_set_heading(
    VectoryFeatureCollectionHandle* handle,
    VectoryEuler heading);
bool vectory_feature_collection_set_global_property(
    VectoryFeatureCollectionHandle* handle,
    const char* key,
    const char* value);
char* vectory_feature_collection_get_global_property(
    const VectoryFeatureCollectionHandle* handle,
    const char* key);
bool vectory_feature_collection_add_point_feature(
    VectoryFeatureCollectionHandle* handle,
    VectoryPoint3 point,
    const char* property_key,
    const char* property_value);
bool vectory_feature_collection_add_segment_feature(
    VectoryFeatureCollectionHandle* handle,
    VectorySegment3 segment,
    const char* property_key,
    const char* property_value);
bool vectory_feature_collection_add_path_feature(
    VectoryFeatureCollectionHandle* handle,
    VectoryPointArrayView points,
    const char* property_key,
    const char* property_value);
bool vectory_feature_collection_add_polygon_feature(
    VectoryFeatureCollectionHandle* handle,
    VectoryPointArrayView points,
    const char* property_key,
    const char* property_value);
VectoryGeometryKind vectory_feature_collection_feature_geometry_kind(
    const VectoryFeatureCollectionHandle* handle,
    size_t index);
char* vectory_feature_collection_feature_property(
    const VectoryFeatureCollectionHandle* handle,
    size_t index,
    const char* key);
bool vectory_feature_collection_feature_point(
    const VectoryFeatureCollectionHandle* handle,
    size_t index,
    VectoryPoint3* out);
bool vectory_feature_collection_feature_segment(
    const VectoryFeatureCollectionHandle* handle,
    size_t index,
    VectorySegment3* out);
size_t vectory_feature_collection_feature_vertex_count(
    const VectoryFeatureCollectionHandle* handle,
    size_t index);
bool vectory_feature_collection_feature_vertex_at(
    const VectoryFeatureCollectionHandle* handle,
    size_t index,
    size_t vertex_index,
    VectoryPoint3* out);

VectoryVectorHandle* vectory_vector_new(
    VectoryPointArrayView boundary,
    VectoryGeo3 datum,
    VectoryEuler heading,
    VectoryCrs crs);
VectoryVectorHandle* vectory_vector_from_file(const char* path);
void vectory_vector_free(VectoryVectorHandle* handle);
bool vectory_vector_to_file(
    const VectoryVectorHandle* handle,
    const char* path,
    VectoryCrs crs);
size_t vectory_vector_element_count(const VectoryVectorHandle* handle);
bool vectory_vector_has_elements(const VectoryVectorHandle* handle);
bool vectory_vector_clear_elements(VectoryVectorHandle* handle);
bool vectory_vector_get_datum(const VectoryVectorHandle* handle, VectoryGeo3* out);
bool vectory_vector_set_datum(VectoryVectorHandle* handle, VectoryGeo3 datum);
bool vectory_vector_get_heading(const VectoryVectorHandle* handle, VectoryEuler* out);
bool vectory_vector_set_heading(VectoryVectorHandle* handle, VectoryEuler heading);
bool vectory_vector_get_crs(const VectoryVectorHandle* handle, VectoryCrs* out);
bool vectory_vector_set_crs(VectoryVectorHandle* handle, VectoryCrs crs);
size_t vectory_vector_field_boundary_vertex_count(const VectoryVectorHandle* handle);
bool vectory_vector_field_boundary_vertex_at(
    const VectoryVectorHandle* handle,
    size_t index,
    VectoryPoint3* out);
bool vectory_vector_set_global_property(
    VectoryVectorHandle* handle,
    const char* key,
    const char* value);
char* vectory_vector_get_global_property(
    const VectoryVectorHandle* handle,
    const char* key,
    const char* default_value);
bool vectory_vector_add_point(
    VectoryVectorHandle* handle,
    VectoryPoint3 point,
    const char* kind);
bool vectory_vector_add_segment(
    VectoryVectorHandle* handle,
    VectorySegment3 segment,
    const char* kind);
bool vectory_vector_add_path(
    VectoryVectorHandle* handle,
    VectoryPointArrayView points,
    const char* kind);
bool vectory_vector_add_polygon(
    VectoryVectorHandle* handle,
    VectoryPointArrayView points,
    const char* kind);
char* vectory_vector_element_kind(
    const VectoryVectorHandle* handle,
    size_t index);
VectoryGeometryKind vectory_vector_element_geometry_kind(
    const VectoryVectorHandle* handle,
    size_t index);
bool vectory_vector_element_point(
    const VectoryVectorHandle* handle,
    size_t index,
    VectoryPoint3* out);

#ifdef __cplusplus
}
#endif

#endif
