use super::native_collection::{NativeCollection, NativeProperty};
use super::native_object::NativeObject;
use crate::core::reader::IsarReader;
use serde_json::Value;
use std::borrow::Cow;

pub struct NativeReader<'a> {
    id: i64,
    object: NativeObject<'a>,
    collection: &'a NativeCollection,
    all_collections: &'a [NativeCollection],
}

impl<'a> NativeReader<'a> {
    pub fn new(
        id: i64,
        object: NativeObject<'a>,
        collection: &'a NativeCollection,
        all_collections: &'a [NativeCollection],
    ) -> Self {
        NativeReader {
            id,
            object,
            collection,
            all_collections,
        }
    }
}

impl<'a> IsarReader for NativeReader<'a> {
    type ObjectReader<'b> = NativeReader<'b> where 'a: 'b;

    type ListReader<'b> = NativeListReader<'b> where 'a: 'b;

    fn read_id(&self) -> i64 {
        self.id
    }

    fn read_byte(&self, index: usize) -> u8 {
        let property = &self.collection.properties[index];
        self.object.read_byte(property.offset as usize)
    }

    fn read_int(&self, index: usize) -> i32 {
        let property = &self.collection.properties[index];
        self.object.read_int(property.offset as usize)
    }

    fn read_float(&self, index: usize) -> f32 {
        let property = &self.collection.properties[index];
        self.object.read_float(property.offset as usize)
    }

    fn read_long(&self, index: usize) -> i64 {
        let property = &self.collection.properties[index];
        self.object.read_long(property.offset as usize)
    }

    fn read_double(&self, index: usize) -> f64 {
        let property = &self.collection.properties[index];
        self.object.read_double(property.offset as usize)
    }

    fn read_string(&self, index: usize) -> Option<&'a str> {
        let property = &self.collection.properties[index];
        self.object.read_string(property.offset as usize)
    }

    fn read_blob(&self, index: usize) -> Option<Cow<'a, [u8]>> {
        let property = &self.collection.properties[index];
        self.object
            .read_bytes(property.offset as usize)
            .map(Cow::Borrowed)
    }

    fn read_json(&self, index: usize) -> Option<Cow<'a, Value>> {
        let property = &self.collection.properties[index];
        let value = self.object.read_json(property.offset as usize);
        value.map(Cow::Owned)
    }

    fn read_object(&self, index: usize) -> Option<Self::ObjectReader<'_>> {
        let property = &self.collection.properties[index];
        let object = self.object.read_object(property.offset as usize)?;

        let collection_index = property.embedded_collection_index.unwrap();
        let collection = &self.all_collections[collection_index as usize];
        Some(NativeReader {
            id: i64::MIN,
            object,
            collection,
            all_collections: self.all_collections,
        })
    }

    fn read_list(&self, index: usize) -> Option<(Self::ListReader<'_>, usize)> {
        let property = self.collection.properties[index];
        let (object, length) = self
            .object
            .read_list(property.offset as usize, property.data_type)?;
        Some((
            NativeListReader {
                object,
                property,
                all_collections: self.all_collections,
            },
            length,
        ))
    }
}

pub struct NativeListReader<'a> {
    object: NativeObject<'a>,
    property: NativeProperty,
    all_collections: &'a [NativeCollection],
}

impl<'a> IsarReader for NativeListReader<'a> {
    type ObjectReader<'b> = NativeReader<'b> where 'a: 'b;

    type ListReader<'b> = NativeListReader<'b> where 'a: 'b;

    fn read_id(&self) -> i64 {
        panic!("Cannot read id from list")
    }

    fn read_byte(&self, index: usize) -> u8 {
        self.object.read_byte(index)
    }

    fn read_int(&self, index: usize) -> i32 {
        self.object.read_int(index * 4)
    }

    fn read_float(&self, index: usize) -> f32 {
        self.object.read_float(index * 4)
    }

    fn read_long(&self, index: usize) -> i64 {
        self.object.read_long(index * 8)
    }

    fn read_double(&self, index: usize) -> f64 {
        self.object.read_double(index * 8)
    }

    fn read_string(&self, index: usize) -> Option<&'a str> {
        self.object.read_string(index * 6)
    }

    fn read_blob(&self, _index: usize) -> Option<Cow<'_, [u8]>> {
        panic!("Nested lists are not supported")
    }

    fn read_json(&self, index: usize) -> Option<Cow<'a, Value>> {
        let valye = self.object.read_json(index * 6);
        valye.map(Cow::Owned)
    }

    fn read_object(&self, index: usize) -> Option<Self::ObjectReader<'_>> {
        let object = self.object.read_object(index * 6)?;
        let collection_index = self.property.embedded_collection_index.unwrap();
        let collection = &self.all_collections[collection_index as usize];
        Some(NativeReader {
            id: i64::MIN,
            object,
            collection,
            all_collections: self.all_collections,
        })
    }

    fn read_list(&self, _index: usize) -> Option<(Self::ListReader<'_>, usize)> {
        panic!("Nested lists are not supported")
    }
}
