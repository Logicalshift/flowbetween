use super::property::*;

impl ToCanvasProperties for &[&dyn ToCanvasProperties] {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        let mut result = vec![];

        for prop in self.iter() {
            result.extend(prop.to_properties());
        }

        result
    }
}

impl ToCanvasProperties for Vec<&dyn ToCanvasProperties> {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        let mut result = vec![];

        for prop in self.iter() {
            result.extend(prop.to_properties());
        }

        result
    }
}
