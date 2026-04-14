use std::{collections::HashMap, path::Path};

use concord::Geo;

use crate::{
    Crs, Feature, FeatureCollection, Geometry, Heading, Path3, Point3, Polygon3, Result, Segment3,
    read, write,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub geometry: Geometry,
    pub properties: HashMap<String, String>,
    pub kind: String,
}

impl Element {
    pub fn new(geometry: Geometry, properties: HashMap<String, String>, kind: impl Into<String>) -> Self {
        Self {
            geometry,
            properties,
            kind: kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    field_boundary: Polygon3,
    field_properties: HashMap<String, String>,
    elements: Vec<Element>,
    datum: Geo,
    heading: Heading,
    crs: Crs,
    global_properties: HashMap<String, String>,
}

impl Vector {
    pub fn new(field_boundary: Polygon3, datum: Geo, heading: Heading, crs: Crs) -> Self {
        Self {
            field_boundary,
            field_properties: HashMap::new(),
            elements: Vec::new(),
            datum,
            heading,
            crs,
            global_properties: HashMap::new(),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let collection = read(path)?;
        if collection.features.is_empty() {
            return Err(crate::Error::InvalidGeoJson(
                "Vector::fromFile: No features found in file".into(),
            ));
        }

        let mut field_polygon = None;

        for feature in &collection.features {
            if let Geometry::Polygon(polygon) = &feature.geometry {
                if feature.properties.get("type").is_some_and(|value| value == "field") {
                    field_polygon = Some((polygon.clone(), feature.properties.clone()));
                    break;
                }
            }
        }

        if field_polygon.is_none() {
            for feature in &collection.features {
                if let Geometry::Polygon(polygon) = &feature.geometry {
                    field_polygon = Some((polygon.clone(), feature.properties.clone()));
                    break;
                }
            }
        }

        let Some((field_boundary, field_properties)) = field_polygon else {
            return Err(crate::Error::InvalidGeoJson(
                "Vector::fromFile: No polygon found to use as field boundary".into(),
            ));
        };

        let mut vector = Self::new(field_boundary, collection.datum, collection.heading, Crs::Enu);
        vector.field_properties = field_properties;
        vector.global_properties = collection.global_properties.clone();

        for feature in collection.features {
            let is_explicit_field = feature.properties.get("type").is_some_and(|value| value == "field");
            if !is_explicit_field {
                let kind = feature
                    .properties
                    .get("type")
                    .cloned()
                    .unwrap_or_else(|| "unknown".into());
                vector
                    .elements
                    .push(Element::new(feature.geometry, feature.properties, kind));
            }
        }

        Ok(vector)
    }

    #[allow(non_snake_case)]
    pub fn fromFile(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_file(path)
    }

    pub fn to_file(&self, path: impl AsRef<Path>, output_crs: Crs) -> Result<()> {
        let mut collection = FeatureCollection::new(self.datum, self.heading);
        collection.global_properties = self.global_properties.clone();

        let mut field_properties = self.field_properties.clone();
        field_properties.insert("type".into(), "field".into());
        collection.features.push(Feature {
            geometry: Geometry::Polygon(self.field_boundary.clone()),
            properties: field_properties,
        });

        for element in &self.elements {
            collection.features.push(Feature {
                geometry: element.geometry.clone(),
                properties: element.properties.clone(),
            });
        }

        write(&collection, path, output_crs)
    }

    #[allow(non_snake_case)]
    pub fn toFile(&self, path: impl AsRef<Path>, output_crs: Crs) -> Result<()> {
        self.to_file(path, output_crs)
    }

    pub fn field_boundary(&self) -> &Polygon3 {
        &self.field_boundary
    }

    #[allow(non_snake_case)]
    pub fn getFieldBoundary(&self) -> &Polygon3 {
        self.field_boundary()
    }

    pub fn set_field_boundary(&mut self, boundary: Polygon3) {
        self.field_boundary = boundary;
    }

    pub fn field_properties(&self) -> &HashMap<String, String> {
        &self.field_properties
    }

    #[allow(non_snake_case)]
    pub fn getFieldProperties(&self) -> &HashMap<String, String> {
        self.field_properties()
    }

    pub fn set_field_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.field_properties.insert(key.into(), value.into());
    }

    pub fn remove_field_property(&mut self, key: &str) {
        self.field_properties.remove(key);
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    #[allow(non_snake_case)]
    pub fn elementCount(&self) -> usize {
        self.element_count()
    }

    pub fn has_elements(&self) -> bool {
        !self.elements.is_empty()
    }

    #[allow(non_snake_case)]
    pub fn hasElements(&self) -> bool {
        self.has_elements()
    }

    pub fn clear_elements(&mut self) {
        self.elements.clear();
    }

    pub fn element(&self, index: usize) -> Option<&Element> {
        self.elements.get(index)
    }

    pub fn get_element(&self, index: usize) -> Result<&Element> {
        self.element(index)
            .ok_or_else(|| crate::Error::InvalidGeoJson("Element index out of range".into()))
    }

    #[allow(non_snake_case)]
    pub fn getElement(&self, index: usize) -> Result<&Element> {
        self.get_element(index)
    }

    pub fn add_element(
        &mut self,
        geometry: Geometry,
        kind: impl Into<String>,
        mut properties: HashMap<String, String>,
    ) {
        let kind = kind.into();
        if !kind.is_empty() {
            properties.insert("type".into(), kind.clone());
        }
        self.elements.push(Element::new(geometry, properties, kind));
    }

    pub fn remove_element(&mut self, index: usize) -> Option<Element> {
        if index < self.elements.len() {
            Some(self.elements.remove(index))
        } else {
            None
        }
    }

    pub fn add_point(
        &mut self,
        point: Point3,
        kind: impl Into<String>,
        properties: HashMap<String, String>,
    ) {
        self.add_element(Geometry::Point(point), kind, properties);
    }

    pub fn add_line(
        &mut self,
        line: Segment3,
        kind: impl Into<String>,
        properties: HashMap<String, String>,
    ) {
        self.add_element(Geometry::Segment(line), kind, properties);
    }

    pub fn add_path(
        &mut self,
        path: Path3,
        kind: impl Into<String>,
        properties: HashMap<String, String>,
    ) {
        self.add_element(Geometry::Path(path), kind, properties);
    }

    pub fn add_polygon(
        &mut self,
        polygon: Polygon3,
        kind: impl Into<String>,
        properties: HashMap<String, String>,
    ) {
        self.add_element(Geometry::Polygon(polygon), kind, properties);
    }

    pub fn elements_by_type(&self, kind: &str) -> Vec<&Element> {
        self.elements.iter().filter(|element| element.kind == kind).collect()
    }

    pub fn elements_by_type_cloned(&self, kind: &str) -> Vec<Element> {
        self.elements_by_type(kind).into_iter().cloned().collect()
    }

    #[allow(non_snake_case)]
    pub fn getElementsByType(&self, kind: &str) -> Vec<Element> {
        self.elements_by_type_cloned(kind)
    }

    pub fn points(&self) -> Vec<&Element> {
        self.elements
            .iter()
            .filter(|element| matches!(element.geometry, Geometry::Point(_)))
            .collect()
    }

    #[allow(non_snake_case)]
    pub fn getPoints(&self) -> Vec<Element> {
        self.points().into_iter().cloned().collect()
    }

    pub fn lines(&self) -> Vec<&Element> {
        self.elements
            .iter()
            .filter(|element| matches!(element.geometry, Geometry::Segment(_)))
            .collect()
    }

    #[allow(non_snake_case)]
    pub fn getLines(&self) -> Vec<Element> {
        self.lines().into_iter().cloned().collect()
    }

    pub fn paths(&self) -> Vec<&Element> {
        self.elements
            .iter()
            .filter(|element| matches!(element.geometry, Geometry::Path(_)))
            .collect()
    }

    #[allow(non_snake_case)]
    pub fn getPaths(&self) -> Vec<Element> {
        self.paths().into_iter().cloned().collect()
    }

    pub fn polygons(&self) -> Vec<&Element> {
        self.elements
            .iter()
            .filter(|element| matches!(element.geometry, Geometry::Polygon(_)))
            .collect()
    }

    #[allow(non_snake_case)]
    pub fn getPolygons(&self) -> Vec<Element> {
        self.polygons().into_iter().cloned().collect()
    }

    pub fn filter_by_property(&self, key: &str, value: &str) -> Vec<&Element> {
        self.elements
            .iter()
            .filter(|element| element.properties.get(key).is_some_and(|candidate| candidate == value))
            .collect()
    }

    pub fn datum(&self) -> Geo {
        self.datum
    }

    pub fn set_datum(&mut self, datum: Geo) {
        self.datum = datum;
    }

    pub fn heading(&self) -> Heading {
        self.heading
    }

    pub fn set_heading(&mut self, heading: Heading) {
        self.heading = heading;
    }

    pub fn crs(&self) -> Crs {
        self.crs
    }

    #[allow(non_snake_case)]
    pub fn getCRS(&self) -> Crs {
        self.crs()
    }

    pub fn set_crs(&mut self, crs: Crs) {
        self.crs = crs;
    }

    #[allow(non_snake_case)]
    pub fn setCRS(&mut self, crs: Crs) {
        self.set_crs(crs);
    }

    pub fn set_global_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.global_properties.insert(key.into(), value.into());
    }

    #[allow(non_snake_case)]
    pub fn setGlobalProperty(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.set_global_property(key, value);
    }

    pub fn global_property(&self, key: &str) -> Option<&str> {
        self.global_properties.get(key).map(String::as_str)
    }

    pub fn global_property_or(&self, key: &str, default: &str) -> String {
        self.global_property(key).unwrap_or(default).to_string()
    }

    #[allow(non_snake_case)]
    pub fn getGlobalProperty(&self, key: &str, default: &str) -> String {
        self.global_property_or(key, default)
    }

    pub fn global_properties(&self) -> &HashMap<String, String> {
        &self.global_properties
    }

    pub fn remove_global_property(&mut self, key: &str) {
        self.global_properties.remove(key);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter()
    }

    pub fn begin(&self) -> std::slice::Iter<'_, Element> {
        self.elements.iter()
    }

    pub fn cbegin(&self) -> std::slice::Iter<'_, Element> {
        self.elements.iter()
    }
}

impl<'a> IntoIterator for &'a Vector {
    type Item = &'a Element;
    type IntoIter = std::slice::Iter<'a, Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}
