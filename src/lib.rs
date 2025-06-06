//! ## Robust Rust library for converting JSON objects into CSV rows
//!
//! Given an array of JSON objects or a file that contains JSON objects one after the other, it
//! produces a CSV file with one row per JSON processed. In order to transform a JSON object into a
//! CSV row, this library "flattens" the objects, converting them into equivalent ones without nested
//! objects or arrays. The rules used for flattening objects are configurable, but by default an
//! object like this:
//!
//! ```json
//! {"a": {"b": [1,2,3]}}
//! ```
//!
//! is transformed into the flattened JSON object:
//!
//! ```json
//! {
//!   "a.b.0": 1,
//!   "a.b.1": 2,
//!   "a.b.2": 3
//! }
//! ```
//!
//! and then used to generate the following CSV output:
//! ```csv
//! a.b.0,a.b.1,a.b.2
//! 1,2,3
//! ```
//!
//! ### Configuring output
//!
//! This library relies on
//! [`flatten-json-object`](https://docs.rs/flatten-json-object/latest/flatten_json_object/)
//! for JSON object flattering and [`csv`](https://docs.rs/csv/latest/csv/) for CSV file generation.
//! Please check their respective documentation if you want to adjust how the output looks.
//!
//! ### Notes
//!
//! - How objects are flattened and the CSV format (e.g. the field separator) can be configured.
//! - Each top level object in the input will be transformed into a CSV row.
//! - The headers are sorted alphabetically and are the union of all the keys in all the objects in
//!   the input after they are flattened.
//! - Key collisions after flattening the input will be reported as errors, i.e. if two objects have
//!   keys that should be different but end looking the same after flattening. For example,
//!   flattening a file that contains `{"a": {"b": 1}} {"a.b": 2}` results by default in an error.
//! - Any instance of `{}` (when not a top level object), `[]` or `Null` results in an empty CSV
//!   field.
//!
//! ### Example reading from a `Read` implementer
//!
//!```rust
//!# use std::error::Error;
//!#
//!# fn main() -> Result<(), Box<dyn Error>> {
//!#
//! use csv;
//! use flatten_json_object::ArrayFormatting;
//! use flatten_json_object::Flattener;
//! use json_objects_to_csv::Json2Csv;
//! use std::io::{Read, Write};
//! use std::str;
//!
//! // Anything supported by the `Flattener` object is possible.
//! let flattener = Flattener::new()
//!     .set_key_separator(".")
//!     .set_array_formatting(ArrayFormatting::Surrounded{
//!         start: "[".to_string(),
//!         end: "]".to_string()
//!     })
//!     .set_preserve_empty_arrays(false)
//!     .set_preserve_empty_objects(false);
//!
//! // The output can be anything that implements `Write`. In this example we use a vector but
//! // this could be a `File`.
//! let mut output = Vec::<u8>::new();
//!
//! // Anything that implements `Read`. Usually a file, but we will use a byte array in this example.
//! let input = r#"{"a": {"b": 1}} {"c": [2]} {"d": []} {"e": {}}"#.as_bytes();
//!
//! // The CSV rows that we should get from this input and config. Note that since we are not
//! // preserving empty arrays or objects `d` and `e` are not part of the final headers.
//! // However, an empty row is generate for their object. If empty objects and arrays were
//! // preserved both `e` and `d` would be part of the headers, but their column would be empty.
//! let expected = ["a.b,c[0]", "1,", ",2", ",", ","];
//!
//! // Here we can configure another field separator, like `;` or use any other CSV builder
//! // configuration.
//! let csv_writer = csv::WriterBuilder::new()
//!     .delimiter(b',')
//!     .from_writer(&mut output);
//!
//! Json2Csv::new(flattener).convert_from_reader(input, csv_writer)?;
//!
//! assert_eq!(str::from_utf8(&output)?, expected.join("\n") + "\n");
//!#
//!#     Ok(())
//!# }
//! ```
//!
//! ### Example converting a slice of JSON objects
//!
//!```rust
//!# use std::error::Error;
//!#
//!# fn main() -> Result<(), Box<dyn Error>> {
//!#
//! use csv;
//! use flatten_json_object::ArrayFormatting;
//! use flatten_json_object::Flattener;
//! use json_objects_to_csv::Json2Csv;
//! use serde_json::json;
//! use std::str;
//!
//! // We changed the array formatting and we preserve empty arrays and objects now, compared to
//! // the previous example.
//! let flattener = Flattener::new()
//!     .set_key_separator(".")
//!     .set_array_formatting(ArrayFormatting::Plain)
//!     .set_preserve_empty_arrays(true)
//!     .set_preserve_empty_objects(true);
//!
//! // The output can be anything that implements `Write`. In this example we use a vector but
//! // this could be a `File`.
//! let mut output = Vec::<u8>::new();
//!
//! let input = [
//!     json!({"a": {"b": 1}}),
//!     json!({"c": [2]}),
//!     json!({"d": []}),
//!     json!({"e": {}})
//! ];
//!
//! // This time the separator is `;`
//! let csv_writer = csv::WriterBuilder::new()
//!     .delimiter(b';')
//!     .from_writer(&mut output);
//!
//! // The CSV rows that we should get from this input and config. We are preserving empty arrays
//! // and objects so `d` and `e` are part of the final headers. Since they are empty and no other
//! // object has those headers these columns have no value in any of the rows.
//! let expected = ["a.b;c.0;d;e", "1;;;", ";2;;", ";;;", ";;;"];
//!
//! Json2Csv::new(flattener).convert_from_array(&input, csv_writer)?;
//!
//! assert_eq!(str::from_utf8(&output)?, expected.join("\n") + "\n");
//!#
//!#     Ok(())
//!# }
//! ```
//!
//! ### Example preserving key order
//!
//!```rust
//!# use std::error::Error;
//!#
//!# fn main() -> Result<(), Box<dyn Error>> {
//!#
//! use csv;
//! use flatten_json_object::Flattener;
//! use json_objects_to_csv::Json2Csv;
//! use serde_json::json;
//! use std::str;
//!
//! let flattener = Flattener::new();
//! let mut output = Vec::<u8>::new();
//!
//! let input = [
//!     json!({"price": 2.50, "fruit": "apple"}),
//!     json!({"price": 3.00, "fruit": "banana"})
//! ];
//!
//! let csv_writer = csv::Writer::from_writer(&mut output);
//!
//! // By default, headers are sorted alphabetically: "fruit,price"
//! // With preserve_key_order(true), headers maintain original order: "price,fruit"
//! Json2Csv::new(flattener)
//!     .preserve_key_order(true)
//!     .convert_from_array(&input, csv_writer)?;
//!
//! let expected = ["price,fruit", "2.5,apple", "3.0,banana"];
//! assert_eq!(str::from_utf8(&output)?, expected.join("\n") + "\n");
//!#
//!#     Ok(())
//!# }
//! ```

use flatten_json_object::ArrayFormatting;
use serde_json::{Deserializer, Value};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Write};
use tempfile::tempfile;

pub use csv;
pub use error::Error;
pub use flatten_json_object;

mod error;

/// Collection of headers that can be either sorted (BTreeSet) or ordered (Vec).
#[derive(Clone, Debug)]
enum HeaderCollection {
    /// Headers sorted alphabetically
    Sorted(BTreeSet<String>),
    /// Headers in insertion order
    Ordered(Vec<String>),
}

impl HeaderCollection {
    /// Insert a header into the collection
    fn insert(&mut self, key: String) {
        match self {
            HeaderCollection::Sorted(set) => {
                set.insert(key);
            }
            HeaderCollection::Ordered(vec) => {
                if !vec.contains(&key) {
                    vec.push(key);
                }
            }
        }
    }

    /// Check if the collection is empty
    fn is_empty(&self) -> bool {
        match self {
            HeaderCollection::Sorted(set) => set.is_empty(),
            HeaderCollection::Ordered(vec) => vec.is_empty(),
        }
    }

    /// Get the length of the collection
    fn len(&self) -> usize {
        match self {
            HeaderCollection::Sorted(set) => set.len(),
            HeaderCollection::Ordered(vec) => vec.len(),
        }
    }

    /// Iterate over the headers in their respective order
    fn iter(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        match self {
            HeaderCollection::Sorted(set) => Box::new(set.iter()),
            HeaderCollection::Ordered(vec) => Box::new(vec.iter()),
        }
    }
}

/// Basic struct of this crate. It contains the configuration.Instantiate it and use the method
/// `convert_from_array` or `convert_from_file` to convert the JSON input into a CSV file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Json2Csv {
    /// The flattener that we use internally.
    flattener: flatten_json_object::Flattener,
    /// The flattener provided by the user of the library.
    original_flattener: flatten_json_object::Flattener,
    /// Whether to preserve the original order of keys instead of sorting them alphabetically.
    preserve_key_order: bool,
}

impl Json2Csv {
    /// Creates a JSON to CSV object with the flattening config provided.
    #[must_use]
    pub fn new(flattener: flatten_json_object::Flattener) -> Self {
        // We use replace the separators provided with control characters (which should not be
        // present in normal input) to be able to detect collisions like the one that happens when
        // converting `[{"a": {"b": 1}} {"a.b": 2}]` to CSV with a `.` separator.
        let key_sep = "␝";
        let array_start = "␞";
        let array_end = "␟";
        Json2Csv {
            flattener: match flattener.array_formatting() {
                ArrayFormatting::Plain => flattener.clone().set_key_separator(key_sep),
                ArrayFormatting::Surrounded { start: _, end: _ } => flattener
                    .clone()
                    .set_key_separator(key_sep)
                    .set_array_formatting(ArrayFormatting::Surrounded {
                        start: array_start.to_string(),
                        end: array_end.to_string(),
                    }),
            },
            original_flattener: flattener,
            preserve_key_order: false,
        }
    }

    /// Sets whether to preserve the original order of keys instead of sorting them alphabetically.
    /// 
    /// When set to `true`, headers will appear in the order they are first encountered in the JSON objects.
    /// When set to `false` (default), headers will be sorted alphabetically.
    #[must_use]
    pub const fn preserve_key_order(mut self, preserve: bool) -> Self {
        self.preserve_key_order = preserve;
        self
    }

    /// The library uses internally a different key separator and potentially array formatting
    /// rules compared to what the user specified. This method is used to undo the transformation
    /// before presenting the results to the user.
    fn transform_key(&self, key: &str) -> String {
        let key = key.replace(
            self.flattener.key_separator(),
            self.original_flattener.key_separator(),
        );

        match self.original_flattener.array_formatting() {
            ArrayFormatting::Plain => key,
            ArrayFormatting::Surrounded { start: os, end: oe } => {
                match self.flattener.array_formatting() {
                    ArrayFormatting::Surrounded { start: s, end: e } => {
                        key.replace(e, oe).replace(s, os)
                    }
                    ArrayFormatting::Plain => {
                        unreachable!(
                            "We cloned the original flattener so both should have the same \
                            array formatting enum variant"
                        )
                    }
                }
            }
        }
    }



    /// Collects headers from all objects in a single pass.
    /// Returns a tuple of (transformed_headers, original_headers).
    /// The order of headers depends on the `preserve_key_order` setting.
    fn collect_headers(&self, objects: &[Value]) -> Result<(HeaderCollection, BTreeSet<String>), error::Error> {
        let mut orig_headers = BTreeSet::new();
        let mut headers = if self.preserve_key_order {
            HeaderCollection::Ordered(Vec::new())
        } else {
            HeaderCollection::Sorted(BTreeSet::new())
        };
        
        // Process objects to collect headers in the appropriate order
        for obj in objects {
            let flattened = self.flattener.flatten(obj)?;
            if let Value::Object(map) = flattened {
                for (orig_key, _) in map {
                    let key = self.transform_key(&orig_key);
                    orig_headers.insert(orig_key);
                    headers.insert(key);
                }
            } else {
                unreachable!("Flattening a JSON object always produces a JSON object");
            }
        }
        
        Ok((headers, orig_headers))
    }

    /// Transforms a map using the original-to-transformed key mapping.
    fn transform_map(&self, orig_map: serde_json::value::Map<String, Value>) -> serde_json::value::Map<String, Value> {
        let mut map = serde_json::value::Map::new();
        for (orig_key, value) in orig_map {
            let key = self.transform_key(&orig_key);
            map.insert(key, value);
        }
        map
    }

    /// Flattens each one of the objects in the array slice and transforms each of them into a CSV
    /// row.
    ///
    /// The headers of the CSV are the union of all the keys that result from flattening the
    /// objects in the input.
    ///
    /// # Errors
    /// Will return `Err` if `objects` does not contain actual JSON objects. It will also report an
    /// error if two objects have keys that should be different but end looking the same after
    /// flattening, and if writing the CSV fails.
    pub fn convert_from_array(
        self,
        objects: &[Value],
        mut csv_writer: csv::Writer<impl Write>,
    ) -> Result<(), error::Error> {
        if objects.is_empty() {
            return Ok(());
        }

        // Collect headers in first pass
        let (headers, orig_headers) = self.collect_headers(objects)?;
        
        // If we could not extract headers there is nothing to write to the CSV file
        if headers.is_empty() {
            return Ok(());
        }

        // Check that there are no collisions between flattened keys in different objects
        if headers.len() != orig_headers.len() {
            return Err(Error::FlattenedKeysCollision);
        }

        csv_writer.write_record(headers.iter())?;
        
        // Process objects in streaming fashion - no intermediate collections
        for obj in objects {
            let flattened = self.flattener.flatten(obj)?;
            if let Value::Object(orig_map) = flattened {
                let map = self.transform_map(orig_map);
                csv_writer.write_record(build_record(&headers, map))?;
            } else {
                unreachable!("Flattening a JSON object always produces a JSON object");
            }
        }

        Ok(())
    }

    /// Flattens the JSON objects in the file, transforming each of them into a CSV row.
    ///
    /// The headers of the CSV are the union of all the keys that result from flattening the objects
    /// in the input. The file must contain JSON objects one immediately after the other or
    /// separated by whitespace. Note that it uses a temporary file to store the flattened input,
    /// which is automatically deleted when lo longer necessary.
    ///
    /// # Errors
    /// Will return `Err` if parsing the file fails or if the JSONs there are not objects. It will
    /// also report an error if two objects have keys that should be different but end looking the
    /// same after flattening, and if writing the CSV or to the temporary file fails.
    pub fn convert_from_reader(
        self,
        reader: impl Read,
        mut csv_writer: csv::Writer<impl Write>,
    ) -> Result<(), error::Error> {
        // We have to flatten the JSON objects into a file because it can potentially be a really big
        // stream. We cannot directly convert into CSV because we cannot be sure about all the objects
        // resulting in the same headers.
        let mut tmp_file = BufWriter::new(tempfile()?);

        // The headers are the union of the keys of the flattened objects.
        // We collect the headers with our magic separators, and the headers with the separators that the user requested.
        let mut orig_headers = BTreeSet::<String>::new();
        let mut headers = if self.preserve_key_order {
            HeaderCollection::Ordered(Vec::new())
        } else {
            HeaderCollection::Sorted(BTreeSet::new())
        };

        for obj in Deserializer::from_reader(reader).into_iter::<Value>() {
            let obj = obj?; // Ensure that we can parse the input properly
            let obj = self.flattener.flatten(&obj)?;

            let Value::Object(orig_map) = obj else {
                unreachable!("Flattening a JSON object always produces a JSON object");
            };

            let mut map = BTreeMap::new();
            for (orig_key, value) in orig_map {
                let key = self.transform_key(&orig_key);
                map.insert(key.clone(), value);
                orig_headers.insert(orig_key);
                headers.insert(key);
            }
            serde_json::to_writer(&mut tmp_file, &map)?;
        }

        // If we could not extract headers there is nothing to write to the CSV file
        if headers.is_empty() {
            return Ok(());
        }

        // Check that there are no collisions between flattened keys in different objects
        if headers.len() != orig_headers.len() {
            return Err(Error::FlattenedKeysCollision);
        }

        tmp_file.seek(SeekFrom::Start(0))?;
        let tmp_file = BufReader::new(tmp_file.into_inner()?);

        csv_writer.write_record(headers.iter())?;
        for obj in Deserializer::from_reader(tmp_file).into_iter::<Value>() {
            let Value::Object(map) = obj? else {
                unreachable!("Flattening a JSON object always produces a JSON object");
            };
            csv_writer.write_record(build_record(&headers, map))?;
        }

        Ok(())
    }
}

fn build_record(
    headers: &HeaderCollection,
    mut map: serde_json::Map<String, Value>,
) -> Vec<String> {
    let mut record: Vec<String> = vec![];
    for header in headers.iter() {
        if let Some(val) = map.remove(header) {
            match val {
                Value::String(s) => record.push(s),
                // _ => record.push(val.to_string()),
                Value::Bool(_) | Value::Number(_) => record.push(val.to_string()),
                // Any array or object here must be empty, because it would have been flattened
                // otherwise. In addition, to reach this for arrays and objects the flattener must
                // have been set to preserve them when empty. Makes no sense to add them or `Null`
                // to the CSV output, so we replace them with the empty string.
                Value::Null | Value::Array(_) | Value::Object(_) => record.push(String::new()),
            }
        } else {
            record.push(String::new());
        }
    }
    record
}

#[cfg(test)]
mod tests {
    use super::*;
    use error::Error;
    use flatten_json_object::{ArrayFormatting, Flattener};
    use rstest::rstest;
    use std::str;

    struct ExecutionResult {
        input: Vec<Value>,
        output: String,
    }

    fn execute_expect_err(input: &str, flattener: &Flattener) -> Vec<error::Error> {
        let mut output_from_file = Vec::<u8>::new();
        let csv_writer_from_file = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output_from_file);

        let result_from_file = Json2Csv::new(flattener.clone())
            .convert_from_reader(input.as_bytes(), csv_writer_from_file);

        let input_from_array: Result<Vec<_>, _> =
            Deserializer::from_str(input).into_iter::<Value>().collect();
        let input_from_array = input_from_array.unwrap();

        let mut output_from_array = Vec::<u8>::new();
        let csv_writer_from_array = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output_from_array);
        let result_from_array = Json2Csv::new(flattener.clone())
            .convert_from_array(&input_from_array, csv_writer_from_array);

        // We expect both to produce the same error
        let error_from_file = result_from_file.err().unwrap();
        let error_from_array = result_from_array.err().unwrap();

        vec![error_from_file, error_from_array]
    }

    fn execute(input: &str, flattener: &Flattener) -> ExecutionResult {
        let mut output_from_file = Vec::<u8>::new();
        let csv_writer_from_file = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output_from_file);
        Json2Csv::new(flattener.clone())
            .convert_from_reader(input.as_bytes(), csv_writer_from_file)
            .unwrap();

        let input_from_array: Result<Vec<_>, _> =
            Deserializer::from_str(input).into_iter::<Value>().collect();
        let input_from_array = input_from_array.unwrap();

        let mut output_from_array = Vec::<u8>::new();
        let csv_writer_from_array = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output_from_array);
        Json2Csv::new(flattener.clone())
            .convert_from_array(&input_from_array, csv_writer_from_array)
            .unwrap();

        let output_from_file = str::from_utf8(&output_from_file).unwrap();
        let output_from_array = str::from_utf8(&output_from_array).unwrap();

        assert_eq!(output_from_file, output_from_array);

        ExecutionResult {
            input: input_from_array,
            output: output_from_array.to_string(),
        }
    }

    #[rstest]
    #[case::nesting_and_array(r#"{"a": {"b": 1}}{"c": [2]}"#, &["a.b,c.0", "1,", ",2"])]
    #[case::spaces_end(r#"{"a": {"b": 1}}{"c": [2]}   "#, &["a.b,c.0", "1,", ",2"])]
    #[case::spaces_begin(r#"      {"a": {"b": 1}}{"c": [2]}"#, &["a.b,c.0", "1,", ",2"])]
    #[case::key_repeats_consistently(r#"{"a": 3}{"a": 4}{"a": 5}"#, &["a", "3", "4", "5"])]
    #[case::reordering(r#"{"b": 3, "a": 1}{"a": 4, "b": 2}"#, &["a,b", "1,3", "4,2"])]
    #[case::reordering_with_empty_array(r#"{"b": 3, "a": 1, "c": 0}{"c": [], "a": 4, "b": 2}"#, &["a,b,c", "1,3,0", "4,2,"])]
    #[case::reordering_with_empty_object(r#"{"b": 3, "a": 1, "c": 0}{"c": {}, "a": 4, "b": 2}"#, &["a,b,c", "1,3,0", "4,2,"])]
    #[case::reordering_with_missing(r#"{"b": 3, "a": 1, "c": 0}{"a": 4, "b": 2}"#, &["a,b,c", "1,3,0", "4,2,"])]
    fn simple_input(
        #[case] input: &str,
        #[case] expected: &[&str],
        #[values(true, false)] preserve_empty_arrays: bool,
        #[values(true, false)] preserve_empty_objects: bool,
    ) {
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(preserve_empty_arrays)
            .set_preserve_empty_objects(preserve_empty_objects);
        let result = execute(input, &flattener);
        assert_eq!(result.output, expected.join("\n") + "\n");
    }

    #[test]
    fn duplicated_keys_last_wins() {
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(true)
            .set_preserve_empty_objects(true);
        let result = execute(
            r#"{"a": [1,2,3], "a": {"b": 2}, "c": 1, "c": 2}"#,
            &flattener,
        );
        let expected = &["a.b,c", "2,2"];
        assert_eq!(result.output, expected.join("\n") + "\n");
    }

    /// We use internal separators that later are replaced by the user provided ones.
    /// This checks that the replacement does not make the headers and the data be in a different order.
    #[test]
    fn no_reordering_on_non_default_separators() {
        let flattener = Flattener::new()
            .set_key_separator("]")
            .set_array_formatting(ArrayFormatting::Surrounded {
                start: ".".to_string(),
                end: "".to_string(),
            })
            .set_preserve_empty_arrays(true)
            .set_preserve_empty_objects(true);
        let result = execute(r#"{"a": [1,2,3]} {"a": {"b": 2}}"#, &flattener);
        let expected = &["a.0,a.1,a.2,a]b", "1,2,3,", ",,,2"];
        assert_eq!(result.output, expected.join("\n") + "\n");
    }

    /// An error must be reported when flattening makes two keys in an object look the same.
    #[rstest]
    #[case::in_one_object(r#"{"a": {"b": 1}, "a.b": 2}"#)]
    #[case::in_different_objects(r#"{"a": {"b": 1}}{"a.b": 2}"#)]
    fn error_on_collision(#[case] input: &str) {
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);
        for err in execute_expect_err(input, &flattener) {
            assert!(
                matches!(err, Error::FlattenedKeysCollision),
                "Unexpected error: {}",
                err
            );
        }
    }

    /// An error must be reported when flattening makes two keys in an object look the same, even
    /// when it's due to array formatting.
    #[rstest]
    #[case::in_one_object(r#"{"a[0]": 1, "a": [2]}"#, "[", "]")]
    #[case::in_different_objects(r#"{"a[0]": 1} {"a": [2]}"#, "[", "]")]
    fn error_on_collision_array_formatting(
        #[case] input: &str,
        #[case] start: &str,
        #[case] end: &str,
    ) {
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Surrounded {
                start: start.to_string(),
                end: end.to_string(),
            })
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);
        for err in execute_expect_err(input, &flattener) {
            assert!(
                matches!(err, Error::FlattenedKeysCollision),
                "Unexpected error: {}",
                err
            );
        }
    }

    /// In all those cases there are no headers after flattening the input, so the resulting CSV is
    /// empty.
    #[rstest]
    #[case::empty_string("")]
    #[case::empty_json_doc("{}")]
    #[case::multiple_empty_json_docs("{}{}{}{}")]
    #[case::empty_array(r#"{"a": []}"#)]
    #[case::empty_obj(r#"{"b": {}}"#)]
    #[case::empty_array_obj_and_json_doc(r#"{"a": []} {"b": {}} {}"#)]
    fn empty_csv_when_no_headers(#[case] input: &str) {
        let expected = "";
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);
        let result = execute(input, &flattener);
        assert_eq!(result.output, expected);
    }

    #[rstest]
    #[case::empty_array(r#"{"a": []}"#)]
    #[case::empty_array_extra_obj(r#"{"a": []} {} {}"#)]
    #[case::empty_obj(r#"{"a": {}}"#)]
    #[case::empty_obj_extra_obj(r#"{"a": {}} {}"#)]
    fn preserved_empty(#[case] input: &str) {
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(true)
            .set_preserve_empty_objects(true);
        let result = execute(input, &flattener);

        let mut expected = vec!["a"];

        // Extend the vector with as many rows as objects has the input
        expected.extend(vec![r#""""#; result.input.len()]);

        assert_eq!(result.output, expected.join("\n") + "\n");
    }

    #[rstest]
    #[case::empty_array(r#"{"a": [], "b": 3}"#, &["b", "3"])]
    #[case::empty_array_extra_obj(r#"{"a": [], "b": 3} {} {}"#, &["b", "3", r#""""#, r#""""#])]
    #[case::empty_obj(r#"{"a": {}, "b": 3}"#, &["b", "3"])]
    #[case::empty_obj_extra_obj(r#"{"a": {}} {} {"b": 3} {}"#, &["b", r#""""#, r#""""#, "3", r#""""#])]
    #[case::empty_obj_extra_obj(r#"{"a": {}} {} {"b": 3} {"c": 4}"#, &["b,c", ",", ",", "3,", ",4"])]
    fn not_preserved_empty(#[case] input: &str, #[case] expected: &[&str]) {
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);
        let result = execute(input, &flattener);

        assert_eq!(result.output, expected.join("\n") + "\n");
    }

    #[test]
    fn preserve_key_order_default_is_sorted() {
        let input = r#"{"price": 2.50, "fruit": "apple"}{"price": 3.00, "fruit": "banana"}"#;
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);
        
        // Default behavior should sort headers alphabetically
        let result = execute(input, &flattener);
        let expected = &["fruit,price", "apple,2.5", "banana,3.0"];
        assert_eq!(result.output, expected.join("\n") + "\n");
    }

    #[test]  
    fn preserve_key_order_when_enabled() {
        let input = r#"{"price": 2.50, "fruit": "apple"}{"price": 3.00, "fruit": "banana"}"#;
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);

        // Test with key order preservation enabled
        let mut output_from_array = Vec::<u8>::new();
        let csv_writer_from_array = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output_from_array);

        let input_from_array: Result<Vec<_>, _> =
            Deserializer::from_str(input).into_iter::<Value>().collect();
        let input_from_array = input_from_array.unwrap();

        Json2Csv::new(flattener.clone())
            .preserve_key_order(true)
            .convert_from_array(&input_from_array, csv_writer_from_array)
            .unwrap();

        let output_from_array = std::str::from_utf8(&output_from_array).unwrap();
        
        // Headers should be in original order: price,fruit (not sorted fruit,price)
        let expected = &["price,fruit", "2.5,apple", "3.0,banana"];
        assert_eq!(output_from_array, expected.join("\n") + "\n");
    }

    #[test]
    fn preserve_key_order_with_reader() {
        let input = r#"{"price": 2.50, "fruit": "apple"}{"price": 3.00, "fruit": "banana"}"#;
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);

        // Test with key order preservation using reader
        let mut output = Vec::<u8>::new();
        let csv_writer = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output);

        Json2Csv::new(flattener)
            .preserve_key_order(true)
            .convert_from_reader(input.as_bytes(), csv_writer)
            .unwrap();

        let output = std::str::from_utf8(&output).unwrap();
        
        // Headers should be in original order: price,fruit (not sorted fruit,price)
        let expected = &["price,fruit", "2.5,apple", "3.0,banana"];
        assert_eq!(output, expected.join("\n") + "\n");
    }

    #[test]
    fn preserve_key_order_with_complex_nesting() {
        let input = r#"{"z": {"y": 1}, "a": {"x": 2}}{"z": {"y": 3}, "a": {"x": 4}}"#;
        let flattener = Flattener::new()
            .set_key_separator(".")
            .set_array_formatting(ArrayFormatting::Plain)
            .set_preserve_empty_arrays(false)
            .set_preserve_empty_objects(false);

        // Test with preserve_key_order = true
        let mut output_ordered = Vec::<u8>::new();
        let csv_writer_ordered = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output_ordered);

        let input_array: Result<Vec<_>, _> =
            Deserializer::from_str(input).into_iter::<Value>().collect();
        let input_array = input_array.unwrap();

        Json2Csv::new(flattener.clone())
            .preserve_key_order(true)
            .convert_from_array(&input_array, csv_writer_ordered)
            .unwrap();

        let output_ordered = std::str::from_utf8(&output_ordered).unwrap();
        
        // Headers should be in original order: z.y,a.x (not sorted a.x,z.y)
        let expected_ordered = &["z.y,a.x", "1,2", "3,4"];
        assert_eq!(output_ordered, expected_ordered.join("\n") + "\n");

        // Test with preserve_key_order = false (default) for comparison
        let mut output_sorted = Vec::<u8>::new();
        let csv_writer_sorted = csv::WriterBuilder::new()
            .delimiter(b',')
            .from_writer(&mut output_sorted);

        Json2Csv::new(flattener)
            .preserve_key_order(false)
            .convert_from_array(&input_array, csv_writer_sorted)
            .unwrap();

        let output_sorted = std::str::from_utf8(&output_sorted).unwrap();
        
        // Headers should be sorted: a.x,z.y
        let expected_sorted = &["a.x,z.y", "2,1", "4,3"];
        assert_eq!(output_sorted, expected_sorted.join("\n") + "\n");
    }
}
