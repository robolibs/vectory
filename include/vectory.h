#ifndef VECTORY_H
#define VECTORY_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

enum VectoryCrs
#ifdef __cplusplus
  : uint32_t
#endif // __cplusplus
 {
  VECTORY_CRS_VECTORY_CRS_WGS = 0,
  VECTORY_CRS_VECTORY_CRS_ENU = 1,
};
#ifndef __cplusplus
typedef uint32_t VectoryCrs;
#endif // __cplusplus

enum VectoryGeometryKind
#ifdef __cplusplus
  : uint32_t
#endif // __cplusplus
 {
  VECTORY_GEOMETRY_KIND_VECTORY_GEOMETRY_KIND_NONE = 0,
  VECTORY_GEOMETRY_KIND_VECTORY_GEOMETRY_KIND_POINT = 1,
  VECTORY_GEOMETRY_KIND_VECTORY_GEOMETRY_KIND_SEGMENT = 2,
  VECTORY_GEOMETRY_KIND_VECTORY_GEOMETRY_KIND_PATH = 3,
  VECTORY_GEOMETRY_KIND_VECTORY_GEOMETRY_KIND_POLYGON = 4,
};
#ifndef __cplusplus
typedef uint32_t VectoryGeometryKind;
#endif // __cplusplus

typedef struct VectoryFeatureCollection VectoryFeatureCollection;

typedef struct VectoryVector VectoryVector;

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
  const VectoryPoint3 *ptr;
  uintptr_t len;
} VectoryPointArrayView;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

const char *vectory_last_error_message(void);

void vectory_string_free(char *ptr);

VectoryFeatureCollection *vectory_feature_collection_new(VectoryGeo3 datum, VectoryEuler heading);

void vectory_feature_collection_free(VectoryFeatureCollection *handle);

VectoryFeatureCollection *vectory_feature_collection_read(const char *path);

VectoryFeatureCollection *vectory_feature_collection_from_json(const char *text);

bool vectory_feature_collection_write(const VectoryFeatureCollection *handle,
                                      const char *path,
                                      VectoryCrs crs);

char *vectory_feature_collection_to_json(const VectoryFeatureCollection *handle, VectoryCrs crs);

uintptr_t vectory_feature_collection_feature_count(const VectoryFeatureCollection *handle);

bool vectory_feature_collection_clear_features(VectoryFeatureCollection *handle);

bool vectory_feature_collection_get_datum(const VectoryFeatureCollection *handle, VectoryGeo3 *out);

bool vectory_feature_collection_set_datum(VectoryFeatureCollection *handle, VectoryGeo3 datum);

bool vectory_feature_collection_get_heading(const VectoryFeatureCollection *handle,
                                            VectoryEuler *out);

bool vectory_feature_collection_set_heading(VectoryFeatureCollection *handle, VectoryEuler heading);

bool vectory_feature_collection_set_global_property(VectoryFeatureCollection *handle,
                                                    const char *key,
                                                    const char *value);

char *vectory_feature_collection_get_global_property(const VectoryFeatureCollection *handle,
                                                     const char *key);

bool vectory_feature_collection_add_point_feature(VectoryFeatureCollection *handle,
                                                  VectoryPoint3 point,
                                                  const char *property_key,
                                                  const char *property_value);

bool vectory_feature_collection_add_segment_feature(VectoryFeatureCollection *handle,
                                                    VectorySegment3 segment,
                                                    const char *property_key,
                                                    const char *property_value);

bool vectory_feature_collection_add_path_feature(VectoryFeatureCollection *handle,
                                                 VectoryPointArrayView points,
                                                 const char *property_key,
                                                 const char *property_value);

bool vectory_feature_collection_add_polygon_feature(VectoryFeatureCollection *handle,
                                                    VectoryPointArrayView points,
                                                    const char *property_key,
                                                    const char *property_value);

VectoryGeometryKind vectory_feature_collection_feature_geometry_kind(const VectoryFeatureCollection *handle,
                                                                     uintptr_t index);

char *vectory_feature_collection_feature_property(const VectoryFeatureCollection *handle,
                                                  uintptr_t index,
                                                  const char *key);

bool vectory_feature_collection_feature_point(const VectoryFeatureCollection *handle,
                                              uintptr_t index,
                                              VectoryPoint3 *out);

bool vectory_feature_collection_feature_segment(const VectoryFeatureCollection *handle,
                                                uintptr_t index,
                                                VectorySegment3 *out);

uintptr_t vectory_feature_collection_feature_vertex_count(const VectoryFeatureCollection *handle,
                                                          uintptr_t index);

bool vectory_feature_collection_feature_vertex_at(const VectoryFeatureCollection *handle,
                                                  uintptr_t index,
                                                  uintptr_t vertex_index,
                                                  VectoryPoint3 *out);

VectoryVector *vectory_vector_new(VectoryPointArrayView boundary,
                                  VectoryGeo3 datum,
                                  VectoryEuler heading,
                                  VectoryCrs crs);

VectoryVector *vectory_vector_from_file(const char *path);

void vectory_vector_free(VectoryVector *handle);

bool vectory_vector_to_file(const VectoryVector *handle, const char *path, VectoryCrs crs);

uintptr_t vectory_vector_element_count(const VectoryVector *handle);

bool vectory_vector_has_elements(const VectoryVector *handle);

bool vectory_vector_clear_elements(VectoryVector *handle);

bool vectory_vector_get_datum(const VectoryVector *handle, VectoryGeo3 *out);

bool vectory_vector_set_datum(VectoryVector *handle, VectoryGeo3 datum);

bool vectory_vector_get_heading(const VectoryVector *handle, VectoryEuler *out);

bool vectory_vector_set_heading(VectoryVector *handle, VectoryEuler heading);

bool vectory_vector_get_crs(const VectoryVector *handle, VectoryCrs *out);

bool vectory_vector_set_crs(VectoryVector *handle, VectoryCrs crs);

uintptr_t vectory_vector_field_boundary_vertex_count(const VectoryVector *handle);

bool vectory_vector_field_boundary_vertex_at(const VectoryVector *handle,
                                             uintptr_t index,
                                             VectoryPoint3 *out);

bool vectory_vector_set_global_property(VectoryVector *handle, const char *key, const char *value);

char *vectory_vector_get_global_property(const VectoryVector *handle,
                                         const char *key,
                                         const char *default_value);

bool vectory_vector_add_point(VectoryVector *handle, VectoryPoint3 point, const char *kind);

bool vectory_vector_add_segment(VectoryVector *handle, VectorySegment3 segment, const char *kind);

bool vectory_vector_add_path(VectoryVector *handle, VectoryPointArrayView points, const char *kind);

bool vectory_vector_add_polygon(VectoryVector *handle,
                                VectoryPointArrayView points,
                                const char *kind);

char *vectory_vector_element_kind(const VectoryVector *handle, uintptr_t index);

VectoryGeometryKind vectory_vector_element_geometry_kind(const VectoryVector *handle,
                                                         uintptr_t index);

bool vectory_vector_element_point(const VectoryVector *handle, uintptr_t index, VectoryPoint3 *out);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* VECTORY_H */
